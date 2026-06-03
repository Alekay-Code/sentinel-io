use std::io::ErrorKind;
use std::net::SocketAddr;
use std::net::TcpListener as StdTcpListener;
use std::net::ToSocketAddrs;
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::net::TcpStream;

pub struct TcpListener{
    listener: StdTcpListener,
}

impl TcpListener {
    pub fn bind<A>(addr: A) -> Result<Self, std::io::Error> where A: ToSocketAddrs {
        let listener = StdTcpListener::bind(addr)?;
        listener.set_nonblocking(true)?;
        Ok(TcpListener { listener })
    }

    pub fn accept<'a>(&'a mut self) -> AcceptFuture<'a> {
        AcceptFuture { listener: &mut self.listener }
    }
}

pub struct AcceptFuture<'a> {
    listener: &'a mut StdTcpListener,
}

impl<'a> Future for AcceptFuture<'a> {
    type Output = Result<(TcpStream, SocketAddr), std::io::Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        match this.listener.accept() {
            Ok((stream, addr)) => {
                return Poll::Ready(Ok((TcpStream::from_std_stream(stream), addr)));
            },
            Err(e) => {
                if e.kind() == ErrorKind::WouldBlock {
                    cx.waker().wake_by_ref();
                    return Poll::Pending;
                } else {
                    return Poll::Ready(Err(e));
                }
            }
        }
    }
}
