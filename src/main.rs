use ssh2::Session;
use std::fs::File;
use std::io::prelude::*;
use std::net::TcpStream;
use std::path::Path;
use std::sync::mpsc::channel;

use serde::{Deserialize, Serialize};

// defines how often to sleep between ssh calls
const DELAY: std::time::Duration = std::time::Duration::from_secs(0);

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
    fn ssh_request(&self) -> String {
        let tcp = TcpStream::connect(format!("{}:{}", self.ip, self.port));
        let mut sess = Session::new().unwrap();
        sess.set_tcp_stream(tcp.unwrap());
        sess.handshake().unwrap();
        sess.userauth_pubkey_file(
            &self.username,
            None,
            Path::new(&self.private_key_path),
            None,
        )
        .unwrap();

        let mut channel = sess.channel_session().unwrap();
        channel.exec(&self.command).unwrap();
        let mut s = String::new();
        channel.read_to_string(&mut s).unwrap();
        return s;
    }
}

#[tokio::main]
async fn main() {
    let mut hosts_file = File::open("hosts.json")
        .or(File::open("hosts.json.example"))
        .expect("Expected 'hosts.json' or 'hosts.json.example', but cannot be found in current directory.");
    let mut hosts_file_buff = String::new();
    hosts_file.read_to_string(&mut hosts_file_buff).unwrap();
    let mut processes: Vec<Process> =
        serde_json::from_str(&hosts_file_buff).expect("JSON was not well-formatted.");

    // tx, rx spawn
    let mut tokio_tx_handles = Vec::new();
    let mut tokio_rx_handles = Vec::new();
    while let Some(process) = processes.pop() {
        let (tx, rx) = channel();
        tokio_tx_handles.push(tokio::spawn(async move {
            loop {
                std::thread::sleep(DELAY);
                let _ = tx.send(process.ssh_request());
            }
        }));
        tokio_rx_handles.push(tokio::spawn(async move {
            loop {
                println!("Fetched: {:?}", rx.recv().unwrap());
            }
        }));
    }

    // tx, rx await
    while let Some(tx_handle) = tokio_tx_handles.pop() {
        tx_handle.await.unwrap();
    }
    while let Some(rx_handle) = tokio_rx_handles.pop() {
        let _ = rx_handle.await;
    }
}
