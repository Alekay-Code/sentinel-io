use std::future::Future;

pub trait AsyncRead {
    fn read<'r>(&'r mut self, buf: &'r mut [u8]) -> impl Future<Output = std::io::Result<usize>> + 'r;
}

pub trait AsyncWrite {
    fn write<'r>(&'r mut self, buf: &'r [u8]) -> impl Future<Output = std::io::Result<()>> + 'r;
}
