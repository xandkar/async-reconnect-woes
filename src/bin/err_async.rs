use std::{future::Future, io, pin::pin, time::Duration};

use tokio::{io::AsyncReadExt, net::TcpStream, time::sleep};

struct Worker {
    addr: String,
    stream: Option<TcpStream>,
}

trait Captures<U> {}
impl<T: ?Sized, U> Captures<U> for T {}

impl Worker {
    fn new(addr: &str) -> Self {
        Self {
            addr: addr.to_string(),
            stream: None,
        }
    }

    async fn send(&mut self, msg: &[u8]) -> io::Result<()> {
        self.with(move |stream| {
            use tokio::io::AsyncWriteExt;

            stream.write_all(&msg)
        })
        .await
    }

    // Note, not cancel safe
    fn with<'c, 'b: 'c, 'a: 'b, Fun, Fut>(
        &'a mut self,
        f: Fun,
    ) -> impl Captures<&'a ()> + Future<Output = io::Result<()>>
    where
        Fun: 'c + FnOnce(&'a mut TcpStream) -> Fut,
        Fut: 'c + Future<Output = io::Result<()>>,
        // T: 'static,
    {
        struct InnerStream<'a> {
            stream: &mut Option<TcpStream>,
        }

        async move {
            let new_stream = if self.stream.is_none() {
                let stream = TcpStream::connect(&self.addr).await?;
                Some(stream)
            } else {
                None
            };
            self.stream = self.stream.take().or(new_stream);
            let stream = self.stream.as_mut().unwrap();
            f(stream).await
        }
        /*
        let result = {
            let stream = self.stream.as_mut().unwrap();
            f(stream).await
        };
        if result.is_err() {
            self.stream = None;
        }
        result
        */
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut w = Worker::new("localhost:8000");
    loop {
        let result = w.send(b"foo\n").await;
        eprintln!("[debug] result: {:?}", result);
        sleep(Duration::from_secs(1)).await
    }
}
