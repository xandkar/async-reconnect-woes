use std::{future::Future, io, net::SocketAddr, sync::Arc, time::Duration};

use tokio::{net::TcpStream, sync::Mutex, time::sleep};

struct Worker {
    addr: SocketAddr,
    stream: Option<Arc<Mutex<TcpStream>>>,
}

impl Worker {
    fn new(addr: SocketAddr) -> Self {
        Self { addr, stream: None }
    }

    async fn send(&mut self, msg: &[u8]) -> io::Result<()> {
        self.with(|stream| async move {
            use tokio::io::AsyncWriteExt;

            let mut stream = stream.lock().await;
            stream.write_all(msg).await
        })
        .await
    }

    async fn with<Fun, Fut, T>(&mut self, f: Fun) -> io::Result<T>
    where
        Fun: FnOnce(Arc<Mutex<TcpStream>>) -> Fut,
        Fut: Future<Output = io::Result<T>>,
    {
        if self.stream.is_none() {
            let stream = TcpStream::connect(self.addr).await?;
            self.stream = Some(Arc::new(Mutex::new(stream)));
        }
        let (stream, result) = {
            let stream = self.stream.take().unwrap();
            let result = f(Arc::clone(&stream)).await;
            (stream, result)
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
