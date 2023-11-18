use ssh2::Session;
use std::fs::File;
use std::io::prelude::*;
use std::net::TcpStream;
use std::path::Path;
use std::sync::mpsc::{channel, Sender};

use serde::{Deserialize, Serialize};

// defines how often to sleep between ssh calls
const DELAY: std::time::Duration = std::time::Duration::from_millis(100);

#[derive(Serialize, Deserialize, Debug)]
struct Process {
    username: String,
    private_key_path: String,
    ip: String,
    port: String,
    command: String,
}

#[allow(dead_code)]
impl Process {
    fn new(
        username: String,
        private_key_path: String,
        ip: String,
        port: String,
        command: String,
    ) -> Process {
        return Process {
            username,
            private_key_path,
            ip,
            port,
            command,
        };
    }

    async fn ssh_request(&self) -> Result<String, Box<dyn std::error::Error>> {
        let tcp = TcpStream::connect(format!("{}:{}", self.ip, self.port));
        let mut sess = Session::new()?;
        sess.set_tcp_stream(tcp?);
        sess.handshake()?;
        sess.userauth_pubkey_file(
            &self.username,
            None,
            Path::new(&self.private_key_path),
            None,
        )?;

        let mut channel = sess.channel_session()?;
        channel.exec(&self.command)?;
        let mut s = String::new();
        channel.read_to_string(&mut s)?;
        return Ok(s);
    }
}

#[tokio::main(flavor = "multi_thread", worker_threads = 100)]
async fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    let mut hosts_file = File::open("hosts.json")
        .or(File::open("hosts.json.example"))
        .expect("Expected 'hosts.json' or 'hosts.json.example', but cannot be found in current directory.");
    let mut hosts_file_buff = String::new();
    hosts_file.read_to_string(&mut hosts_file_buff)?;
    let mut processes: Vec<Process> =
        serde_json::from_str(&hosts_file_buff).expect("JSON was not well-formatted.");

    println!("{:?}", processes);
    // tx, rx spawn
    let mut tokio_tx_handles = Vec::new();
    let (tx, rx) = channel();
    while let Some(process) = processes.pop() {
        let tx_clone: Sender<String> = tx.clone();
        tokio_tx_handles.push(tokio::spawn(async move {
            loop {
                std::thread::sleep(DELAY);
                if let Ok(output_data) = process.ssh_request().await {
                    let _ = tx_clone.send(output_data);
                };
            }
        }));
    }
    let rx_tokio_handle = tokio::spawn(async move {
        loop {
            println!("Fetched: {:?}", rx.recv().unwrap());
        }
    });

    let _ = rx_tokio_handle.await;
    Ok(())
}
