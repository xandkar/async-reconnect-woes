use std::{future::Future, io, net::SocketAddr, time::Duration};

use tokio::{net::TcpStream, time::sleep};

struct Worker {
    addr: SocketAddr,
    stream: Option<TcpStream>,
}

impl Worker {
    fn new(addr: SocketAddr) -> Self {
        Self { addr, stream: None }
    }

    async fn send<'a>(&'a mut self, msg: &[u8]) -> io::Result<()> {
        self.with(|stream: &'a mut TcpStream| {
            use tokio::io::AsyncWriteExt;

            stream.write_all(msg)
        })
        .await
    }

    async fn with<'a, Fun, Fut, T>(&'a mut self, f: Fun) -> io::Result<T>
    where
        Fun: FnOnce(&'a mut TcpStream) -> Fut,
        Fut: Future<Output = io::Result<T>>,
    {
        if self.stream.is_none() {
            let stream = TcpStream::connect(self.addr).await?;
            self.stream = Some(stream);
        }
        let result = {
            let stream = self.stream.as_mut().unwrap();
            f(stream).await
        };
        if result.is_err() {
            self.stream = None;
        }
        result
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut w = Worker::new("127.0.0.1:8000".parse().unwrap());
    loop {
        let result = w.send(b"foo\n").await;
        eprintln!("[debug] result: {:?}", result);
        sleep(Duration::from_secs(1)).await
    }
}
