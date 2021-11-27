use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, ChildStdout, Command};
use tokio::sync::{mpsc, mpsc::{Sender, Receiver}};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct MinerController {
    /// Send with this to cause the minerController to kill the process if it exists
    pub kill_tx: Sender<()>,
    /// Send with this to cause the MinerController to kill the process and then spawn a process
    pub spawn_tx: Sender<()>,
    /// The handle to the child process
    child_handle: Option<Child>,
    /// Contains the output of the miner
    pub buffer: Vec<String>,
}

impl MinerController {
    /// The controller has multiple threads mutating it, which necessitates its references be
    /// encapsulated by Arc<Mutex<>>
    #[tokio::main]
    pub async fn new() -> Arc<Mutex<MinerController>> {
        let (kill_tx, mut kill_rx) = mpsc::channel(2);
        let (spawn_tx, mut spawn_rx) = mpsc::channel(2);
        let controller = Arc::new(Mutex::new(MinerController {
            kill_tx,
            spawn_tx,
            child_handle: None,
            buffer: Vec::new()
        }));

        let controller2 = controller.clone();
        // Starts a thread that kills when receiving the kill signal
        tokio::spawn(async move {
            if let Some(()) = kill_rx.recv().await {
                controller2.lock().await.kill_miner().await;
            }
        });

        let controller3 = controller.clone();
        // Starts a thread that kills and then spawns when receiving the spawn signal
        tokio::spawn(async move {
            if let Some(()) = spawn_rx.recv().await {
                controller3.lock().await.spawn_miner().await;
            }
        });

        controller
    }

    /// This function is run by the spawn_rx on receiving
    async fn spawn_miner(&mut self) {
        self.kill_miner().await;

        println!("Spawning...");
        let mut cmd = Command::new("ping")
            .arg("google.com")
            .stdout(Stdio::piped()) // Can do the same for stderr
            .spawn()
            .expect("cannot spawn");

        let stdout = cmd.stdout.take().expect("no stout");

        //tokio::spawn(async move {kill_after_5(&mut cmd).await});
        // To print out each line
        //while let Some(line) = BufReader::new(stdout).lines().next_line().await.unwrap() {
        let buf = BufReader::new(stdout);
        let mut lines = buf.lines();
        while let Some(line) = lines.next_line().await.unwrap() {
            println!(" > {}", &line);
            self.buffer.push(line);
        }
    }

    /// This function is run by the kill_rx on receiving
    async fn kill_miner(&mut self) {
        println!("Killing");
        if let Some(x) = self.child_handle.as_mut() {
            x.kill().await.expect("Could not kill");
        }
    }
}
