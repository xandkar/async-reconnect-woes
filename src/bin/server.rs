use std::{
    io::Read,
    net::{SocketAddr, TcpListener, TcpStream},
    thread,
};

fn main() {
    let host = "127.0.0.1";
    let port: u16 = 8000;
    let addr: SocketAddr = format!("{host}:{port}").parse().unwrap();
    let listener = TcpListener::bind(addr).unwrap();
    eprintln!("[info] Listening. addr={addr:?}.");
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        thread::spawn(move || handle(stream));
    }
}

fn handle(mut stream: TcpStream) {
    let mut buf = [0; 1024];
    loop {
        eprintln!("[debug] Reading. stream={stream:?}");
        match stream.read(&mut buf) {
            Err(error) => {
                eprintln!("[error] Failed. error={error:?}");
                break;
            }
            Ok(0) => {
                break;
            }
            Ok(n) => {
                let msg = String::from_utf8_lossy(&buf[0..n]);
                eprintln!("[debug] Received. msg={msg:?}");
            }
        }
    }
}
