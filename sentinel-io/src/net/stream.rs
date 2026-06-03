use std::net::TcpStream as StdTcpStream;
use std::net::ToSocketAddrs;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use std::io::{Read, Write};
use std::io::ErrorKind;

use crate::io::{AsyncRead, AsyncWrite};

pub struct TcpStream {
    stream: StdTcpStream,
}

impl TcpStream {
    pub fn connect<A>(addr: A) -> Result<Self, std::io::Error> where A: ToSocketAddrs {
        let stream = StdTcpStream::connect(addr)?;
        stream.set_nonblocking(true)?;
        Ok(TcpStream { stream })
    }

    pub fn from_std_stream(stream: StdTcpStream) -> Self {
        TcpStream { stream }
    }

    // TODO: Really need to consume the current TcpStream?
    pub fn split(self) -> (impl AsyncRead, impl AsyncWrite) {
        let reader = self.stream.try_clone().unwrap();
        let writer = self.stream.try_clone().unwrap();
        return (TcpStream::from_std_stream(reader), TcpStream::from_std_stream(writer));
    }
}

struct ReadFuture<'r> {
    stream: &'r mut StdTcpStream,
    buf: &'r mut [u8],
}

impl<'r> Future for ReadFuture<'r> {
    type Output = std::io::Result<usize>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        match this.stream.read(this.buf) {
            Ok(n) => Poll::Ready(Ok(n)),
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                cx.waker().wake_by_ref();
                Poll::Pending
            }
            Err(e) => Poll::Ready(Err(e)),
        }
    }
}

impl AsyncRead for TcpStream {
    fn read<'r>(&'r mut self, buf: &'r mut [u8]) -> impl Future<Output = std::io::Result<usize>> + 'r {
        ReadFuture { stream: &mut self.stream, buf }
    }
}

struct WriteFuture<'r> {
    stream: &'r mut StdTcpStream,
    buf: &'r [u8],
}

impl<'r> Future for WriteFuture<'r> {
    type Output = std::io::Result<()>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        match this.stream.write_all(this.buf) {
            Ok(_) => Poll::Ready(Ok(())),
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                cx.waker().wake_by_ref();
                Poll::Pending
            }
            Err(e) => Poll::Ready(Err(e)),
        }
    }
}

impl AsyncWrite for TcpStream {
    fn write<'r>(&'r mut self, buf: &'r [u8]) -> impl Future<Output = std::io::Result<()>> + 'r {
        WriteFuture { stream: &mut self.stream, buf }
    }
}
