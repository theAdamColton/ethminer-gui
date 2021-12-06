use console::strip_ansi_codes;
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, ChildStdout, Command};
use tokio::sync::{broadcast, oneshot, Mutex};
use tokio::sync::{mpsc, mpsc::Sender};
use tokio::time::sleep;

use crate::miner_settings::MinerSettings;

/// Async controller for the child mining process.
/// Interaction with MinerController is done via tokio channels
pub struct MinerController {
    /// Send with this to cause the minerController to kill the process if it exists
    pub kill_tx: Sender<()>,
    /// Send with this to cause the MinerController to kill the process and then spawn a process
    pub spawn_tx: Sender<Arc<MinerSettings>>,
    /// Send to this when the buffer has been updated, and the view should redraw
    /// Subscribe to this to get the send updates
    pub updated_tx: tokio::sync::broadcast::Sender<()>,
    /// Send to this when a recoverable error has been encountered
    /// Subscribe to this to get informatino on recoverable errors
    /// With the error message string
    pub error_tx: tokio::sync::broadcast::Sender<&'static str>,
    /// The handle to the child process. This only set to None when the child is killed intentionally.
    child_handle: Option<Child>,
    /// Contains the output of the miner as a Vec of the lines
    pub buffer: Arc<Mutex<Vec<String>>>,
}

impl MinerController {
    /// The controller has multiple threads mutating it, which necessitates its references be
    /// encapsulated by Arc<Mutex<>>
    pub fn new() -> Arc<Mutex<MinerController>> {
        let (kill_tx, mut kill_rx) = mpsc::channel(2);
        let (spawn_tx, mut spawn_rx) = mpsc::channel(2);
        let (updated_tx, _) = tokio::sync::broadcast::channel(2);
        let (error_tx, _) = tokio::sync::broadcast::channel(10);

        let controller = Arc::new(Mutex::new(MinerController {
            kill_tx,
            spawn_tx,
            updated_tx: updated_tx.clone(),
            error_tx: error_tx.clone(),
            child_handle: None,
            buffer: Arc::new(Mutex::new(Vec::new())),
        }));

        let controller2 = controller.clone();
        // Starts a thread that kills when receiving the kill signal
        tokio::spawn(async move {
            loop {
                if let Some(()) = kill_rx.recv().await {
                    println!("recv kill");
                    controller2.lock().await.kill_miner().await;
                }
            }
        });

        let controller3 = controller.clone();
        // Starts a thread that kills and then spawns when receiving the spawn signal
        tokio::spawn(async move {
            loop {
                if let Some(miner_settings) = spawn_rx.recv().await {
                    println!("recv spawn");
                    {
                        let mut mc = controller3.lock().await;
                        let miner_setttings_clone = miner_settings.clone();

                        if mc.spawn_miner(miner_settings).await {
                            let controller = controller3.clone();
                            tokio::spawn(async move {
                                MinerController::spawn_child_exited_checker(controller, miner_setttings_clone).await;
                            });

                            mc.update_buffer(updated_tx.clone()).await;
                        }
                    }
                }
            }
        });

        controller
    }

    /// Checks every few seconds if the child process has exited
    /// If it has, it will send on the child_died_tx channel, and exit
    async fn spawn_child_exited_checker(controller: Arc<Mutex<MinerController>>, miner_settings: Arc<MinerSettings>) {
        loop {
            sleep(tokio::time::Duration::from_secs(7)).await;
            println!("checking if child died...",);
            let mut mc = controller.lock().await;
            match mc.child_handle.as_mut() {
                // If child is Some, child has not been killed intentionally
                Some(child) => {
                    match child.try_wait() {
                        Ok(option_exit) => {
                            if option_exit.is_some() {
                                // The child has exited, without being killed intentionally
                                println!("Miner has exited unexpectedly!");
                                {
                                    let mut buf = mc.buffer.lock().await;
                                    buf.push("".to_string());
                                    buf.push("***** Miner Crashed! *****".to_string());
                                    buf.push("***** Restarting.... *****".to_string());
                                }
                                mc.child_handle = None;
                                mc.spawn_tx.send(miner_settings).await.unwrap();
                                return;
                            }
                        }
                        Err(_) => {}
                    }
                }
                // If child is None, the child was killed intentionally
                None => {
                    return;
                }
            }
        }
    }

    /// This function is run by the spawn_rx on receiving
    /// returns true if the child was spawned
    async fn spawn_miner(&mut self, miner_settings: Arc<MinerSettings>) -> bool {
        self.kill_miner().await;

        println!("Spawning...");
        let cmd = Command::new(miner_settings.bin_path.to_owned())
            .args(miner_settings.render())
            .stdout(Stdio::piped())
            .spawn();

        match cmd {
            Ok(child) => {
                self.child_handle = Some(child);
                true
            }
            Err(error) => {
                // TODO more extensive error matching with specific message for
                // missing executable etc.

                println!("Error spawning: {:?}", error);
                self.error_tx
                    .send("Error spawing ethminer!")
                    .expect("Failed to send error message");
                false
            }
        }
    }

    #[allow(unused_must_use)]
    /// This function is run by spawn_miner, and starts a task that appends
    /// the output of the child_handle process to the output buffer
    async fn update_buffer(&mut self, updated_tx: tokio::sync::broadcast::Sender<()>) {
        if let Some(child_handle) = self.child_handle.as_mut() {
            let stdout: ChildStdout = child_handle.stdout.take().expect("No child stdout");

            let buf = BufReader::new(stdout);
            let mut lines = buf.lines();
            let out = self.buffer.clone();

            // Spawns a thread to read the lines from the buffer as they
            // are made available
            tokio::spawn(async move {
                while let Some(line) = lines.next_line().await.unwrap() {
                    println!(" > {}", &line);
                    let mut o = out.lock().await;
                    o.push(strip_ansi_codes(&line).to_string());
                    if o.len() > 200 {
                        o.remove(0);
                    }
                    // I don't care if this fails if the rx is not recieving
                    updated_tx.send(());
                }
            });
        }
    }

    /// This function is run by the kill_rx on receiving
    async fn kill_miner(&mut self) {
        println!("kill_miner()");
        if let Some(x) = self.child_handle.as_mut() {
            println!("Killing");
            x.kill().await.expect("Could not kill");
            self.child_handle = None;
            {
                let mut buf = self.buffer.lock().await;
                buf.push("".to_string());
                buf.push("***** Killed miner *****".to_string());
            }
        }
    }
}
