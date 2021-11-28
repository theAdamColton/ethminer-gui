use std::process::Stdio;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, ChildStdout, Command};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio::sync::{
    mpsc,
    mpsc::{Receiver, Sender},
};
use tokio::time::{sleep, Duration};

pub struct MinerController {
    /// Send with this to cause the minerController to kill the process if it exists
    pub kill_tx: Sender<()>,
    /// Send with this to cause the MinerController to kill the process and then spawn a process
    pub spawn_tx: Sender<()>,
    /// Send to this when the buffer has been updated, and the view should redraw
    /// Subscribe to this to get the send updates
    pub updated_tx: tokio::sync::broadcast::Sender<()>,
    /// The handle to the child process
    child_handle: Option<Child>,
    /// Contains the output of the miner
    pub buffer: Arc<Mutex<Vec<String>>>,
}

impl MinerController {
    /// The controller has multiple threads mutating it, which necessitates its references be
    /// encapsulated by Arc<Mutex<>>
    pub fn new() -> Arc<Mutex<MinerController>> {
        let (kill_tx, mut kill_rx) = mpsc::channel(2);
        let (spawn_tx, mut spawn_rx) = mpsc::channel(2);
        let (updated_tx, _) = tokio::sync::broadcast::channel(2);

        let controller = Arc::new(Mutex::new(MinerController {
            kill_tx,
            spawn_tx,
            updated_tx: updated_tx.clone(),
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
                if let Some(()) = spawn_rx.recv().await {
                    println!("recv spawn");
                    {
                        let mut mc = controller3.lock().await;
                        mc.spawn_miner().await;
                        mc.update_buffer(updated_tx.clone()).await;
                        println!("Handle returned");
                    }
                }
            }
        });

        controller
    }

    /// This function is run by the spawn_rx on receiving
    async fn spawn_miner(&mut self) {
        self.kill_miner().await;

        println!("Spawning...");
        let cmd = Command::new("ping")
            .arg("google.com")
            .stdout(Stdio::piped()) // Can do the same for stderr
            .spawn()
            .expect("cannot spawn");

        self.child_handle = Some(cmd);
    }

    /// This is a blocking function that will update the buffer based on the
    /// output of the child_handle process
    /// This function returrns a handle to the task that will return the sender so
    /// it can be reused
    async fn update_buffer(&mut self, updated_tx: tokio::sync::broadcast::Sender<()>) {
        if let Some(child_handle) = self.child_handle.as_mut() {
            let stdout: ChildStdout = child_handle.stdout.take().expect("No child stdout");

            let buf = BufReader::new(stdout);
            let mut lines = buf.lines();
            let out = self.buffer.clone();

            // Spawns a thread to read the lines from the buffer as they 
            // are made available
            let handle = tokio::spawn(async move {
                while let Some(line) = lines.next_line().await.unwrap() {
                    println!(" > {}", &line);
                    out.lock().await.push(line);
                    updated_tx.send(()).expect("Failed to transmit update channel");
                }
                return updated_tx;
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
        }
    }
}
