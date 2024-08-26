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

    async fn send(&mut self, msg: &[u8]) -> io::Result<()> {
        self.with(|mut stream| async {
            use tokio::io::AsyncWriteExt;

            let result = stream.write_all(msg).await;
            (stream, result)
        })
        .await
    }

    async fn with<Fun, Fut, T>(&mut self, f: Fun) -> io::Result<T>
    where
        Fun: FnOnce(TcpStream) -> Fut,
        Fut: Future<Output = (TcpStream, io::Result<T>)>,
    {
        if self.stream.is_none() {
            let stream = TcpStream::connect(self.addr).await?;
            self.stream = Some(stream);
        }
        let (stream, result) = {
            let stream = self.stream.take().unwrap_or_else(|| {
                unreachable!("We already ensured above that we have some stream.")
            });
            f(stream).await
        };
        match result {
            Ok(_) => self.stream = Some(stream),
            Err(_) => self.stream = None,
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
