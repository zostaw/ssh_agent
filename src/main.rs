use ssh2::Session;
use std::io::prelude::*;
use std::net::TcpStream;
use std::path::Path;
use std::sync::mpsc::channel;

const DELAY: std::time::Duration = std::time::Duration::from_secs(1);

struct Process<'a> {
    username: &'a str,
    private_key_path: &'a str,
    ip: &'a str,
    port: &'a str,
    command: &'a str,
}

impl<'a> Process<'a> {
    fn new(
        username: &'a str,
        private_key_path: &'a str,
        ip: &'a str,
        port: &'a str,
        command: &'a str,
    ) -> Process<'a> {
        return Process {
            username,
            private_key_path,
            ip,
            port,
            command,
        };
    }
    async fn ssh_request(&self) -> String {
        let tcp = TcpStream::connect(format!("{}:{}", self.ip, self.port));
        let mut sess = Session::new().unwrap();
        sess.set_tcp_stream(tcp.unwrap());
        sess.handshake().unwrap();
        sess.userauth_pubkey_file(self.username, None, Path::new(self.private_key_path), None)
            .unwrap();

        let mut channel = sess.channel_session().unwrap();
        channel.exec(self.command).unwrap();
        let mut s = String::new();
        channel.read_to_string(&mut s).unwrap();
        return s;
    }
}

#[tokio::main]
async fn main() {
    let mut processes: Vec<Process> = Vec::new();
    let mut tokio_tx_handles = Vec::new();
    let mut tokio_rx_handles = Vec::new();

    processes.push(Process::new(
        "zostaw".into(),
        "/Users/zostaw/.ssh/main_key".into(),
        "192.168.1.171".into(),
        "22".into(),
        "ls -latr".into(),
    ));
    processes.push(Process::new(
        "zostaw".into(),
        "/Users/zostaw/.ssh/main_key".into(),
        "192.168.1.171".into(),
        "22".into(),
        "ps -ef | grep ssh".into(),
    ));

    // tx, rx spawn
    while let Some(process) = processes.pop() {
        let (tx, rx) = channel();
        tokio_tx_handles.push(tokio::spawn(async move {
            loop {
                std::thread::sleep(DELAY);
                let _ = tx.send(process.ssh_request().await);
            }
        }));
        tokio_rx_handles.push(tokio::spawn(async move {
            loop {
                println!("{:?}", rx.recv().unwrap());
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
