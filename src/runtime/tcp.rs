use crate::runtime::{Interest, EXECUTOR};
use crate::TcpStream;
use crate::{Shutdown, SocketAddr, ToSocketAddrs};
use std::io;
use std::os::wasi::io::AsRawFd;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

pub struct AsyncTcpStream {
    inner: TcpStream,
}

impl AsyncTcpStream {
    pub fn connect<A: ToSocketAddrs>(addrs: A) -> io::Result<AsyncTcpStream> {
        let inner = TcpStream::connect(addrs)?;
        inner.set_nonblocking(true)?;
        EXECUTOR.with(|ex| {
            ex.reactor.borrow_mut().add(inner.as_raw_fd()).unwrap();
        });
        Ok(Self { inner })
    }

    pub fn shutdown(&self, how: Shutdown) -> io::Result<()> {
        self.inner.shutdown(how)
    }

    /// Get peer address.
    pub fn peer_addr(&self) -> io::Result<SocketAddr> {
        self.inner.peer_addr()
    }

    /// Get local address.
    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.inner.local_addr()
    }

    pub fn set_nonblocking(&self, nonblocking: bool) -> io::Result<()> {
        self.inner.set_nonblocking(nonblocking)
    }

    fn poll_write_priv(&mut self, cx: &mut Context<'_>, buf: &[u8]) -> Poll<io::Result<usize>> {
        let this = self;
        match std::io::Write::write(&mut this.inner, buf) {
            Ok(ret) => return Poll::Ready(Ok(ret)),
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                EXECUTOR.with(|ex| {
                    ex.reactor
                        .borrow_mut()
                        .modify(this.inner.as_raw_fd(), Interest::Write, cx)
                });
                Poll::Pending
            }
            Err(e) => return Poll::Ready(Err(e)),
        }
    }

    fn poll_read_priv(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let this = &mut *self;
        let mut inner = || {
            let b = unsafe {
                &mut *(buf.unfilled_mut() as *mut [std::mem::MaybeUninit<u8>] as *mut [u8])
            };
            match std::io::Read::read(&mut this.inner, b) {
                Ok(ret) => return Poll::Ready(Ok(ret)),
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    EXECUTOR.with(|ex| {
                        ex.reactor
                            .borrow_mut()
                            .modify(this.inner.as_raw_fd(), Interest::Read, cx)
                    });
                    Poll::Pending
                }
                Err(e) => return Poll::Ready(Err(e)),
            }
        };

        let n = match inner()? {
            Poll::Ready(t) => t,
            Poll::Pending => return Poll::Pending,
        };

        unsafe {
            buf.assume_init(n);
            buf.advance(n);
        }
        return Poll::Ready(Ok(()));
    }
}

impl AsyncRead for AsyncTcpStream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        self.poll_read_priv(cx, buf)
    }
}

impl AsyncWrite for AsyncTcpStream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
        self.poll_write_priv(cx, buf)
    }
    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        Poll::Ready(Ok(()))
    }
    fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        self.shutdown(Shutdown::Both)?;
        Poll::Ready(Ok(()))
    }
}
