use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;

pub fn tcp_loopback() -> Result<(), String> {
    let listener = TcpListener::bind("127.0.0.1:0").map_err(|e| e.to_string())?;
    let addr = listener.local_addr().map_err(|e| e.to_string())?;

    let client_thread = thread::spawn(move || {
        std::net::TcpStream::connect(addr)
    });

    let (mut server, peer) = listener.accept().map_err(|e| e.to_string())?;
    let mut client = client_thread
        .join()
        .map_err(|_| "client thread panicked".to_string())?
        .map_err(|e| e.to_string())?;

    // server → client
    server.write_all(b"ping").map_err(|e| e.to_string())?;
    let mut buf = [0u8; 4];
    client.read_exact(&mut buf).map_err(|e| e.to_string())?;
    if &buf != b"ping" {
        return Err(format!("expected 'ping', got {:?}", buf));
    }

    // client → server
    client.write_all(b"pong").map_err(|e| e.to_string())?;
    client.shutdown(std::net::Shutdown::Write).map_err(|e| e.to_string())?;
    server.read_exact(&mut buf).map_err(|e| e.to_string())?;
    if &buf != b"pong" {
        return Err(format!("expected 'pong', got {:?}", buf));
    }

    drop((server, peer)); // explicit for clarity
    Ok(())
}
