use sentinel_io as sentinel;
use sentinel::io::{AsyncRead, AsyncWrite};
use sentinel::net::{TcpListener, TcpStream};
use std::net::SocketAddr;

const SERVER_ADDR: &str = "127.0.0.1:8000";

async fn handle_connection(mut conn: TcpStream, addr: SocketAddr) {
    println!("[{addr}]: New client");
    let mut buffer = [0u8; 1024];

    loop {
        let n = conn.read(&mut buffer).await.unwrap();
        if n > 0 {
            let bytes = &buffer[0..n];
            let msg = String::from_utf8_lossy(bytes).to_string();
            let msg = msg.trim();
            println!("[{addr}] -> {msg}");

            println!("{msg} -> [{addr}]");
            let _ = conn.write(bytes).await;

        } else {
            // connecton close
            println!("[{addr}]: Connection close");
            return
        }
    }
}

#[sentinel::main]
async fn main() {
    let mut server = TcpListener::bind(SERVER_ADDR).unwrap();

    println!("Server listening on {SERVER_ADDR}");
    loop {
        let (client, addr) = server.accept().await.unwrap();
        sentinel::spawn(handle_connection(client, addr));;
    }
}
