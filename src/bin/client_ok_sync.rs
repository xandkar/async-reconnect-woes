use std::{
    io,
    net::{SocketAddr, TcpStream},
    thread::sleep,
    time::Duration,
};

struct Worker {
    addr: SocketAddr,
    stream: Option<TcpStream>,
}

impl Worker {
    fn new(addr: SocketAddr) -> Self {
        Self { addr, stream: None }
    }

    fn send(&mut self, msg: &[u8]) -> io::Result<()> {
        self.with(|stream| {
            use std::io::Write;
            stream.write_all(msg)
        })
    }

    fn with<F, T>(&mut self, f: F) -> io::Result<T>
    where
        F: FnOnce(&mut TcpStream) -> io::Result<T>,
    {
        if self.stream.is_none() {
            let stream = TcpStream::connect(self.addr)?;
            self.stream = Some(stream);
        }
        let stream = self.stream.as_mut().unwrap_or_else(|| unreachable!());
        let result = f(stream);
        if result.is_err() {
            self.stream = None;
        }
        result
    }
}

fn main() -> io::Result<()> {
    let mut w = Worker::new("127.0.0.1:8000".parse().unwrap());
    loop {
        let result = w.send(b"foo\n");
        eprintln!("[debug] result: {:?}", result);
        sleep(Duration::from_secs(1))
    }
}
