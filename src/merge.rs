use std::path::Path;
use tokio::io::{AsyncRead, AsyncWrite};
use futures::task::{Context, Poll};
use tokio::macros::support::Pin;
use futures::io::Error;
use futures::Stream;
use pin_project::pin_project;

#[cfg(target_os = "windows")]
#[derive(Debug)]
pub struct UnixListener {
    inner: crate::windows::UnixListener
}

#[cfg(not(target_os = "windows"))]
#[derive(Debug)]
pub struct UnixListener {
    inner: tokio::net::UnixListener
}

impl UnixListener {
    #[cfg(target_os = "windows")]
    pub fn bind<P: AsRef<Path>>(bind_path: P) -> std::io::Result<Self> {
        crate::listener::UnixListener::bind(bind_path)
            .map(|inner| UnixListener { inner })
    }

    #[cfg(not(target_os = "windows"))]
    pub fn bind<P: AsRef<Path>>(bind_path: P) -> std::io::Result<Self> {
        tokio::net::UnixListener::bind(bind_path)
            .map(|inner| UnixListener { inner })
    }

    #[cfg(target_os = "windows")]
    pub fn incoming(self) -> crate::incoming::Incoming {
        crate::incoming::Incoming::new(self.inner)
    }

    #[cfg(not(target_os = "windows"))]
    pub fn incoming<'a>(&'a mut self) -> tokio::net::unix::Incoming<'a> {
        self.inner.incoming()
    }

}

impl Stream for UnixListener {
    #[cfg(target_os = "windows")]
    type Item = (UnixStream, Option<mio_uds_windows::net::SocketAddr>);
    #[cfg(not(target_os = "windows"))]
    type Item = (UnixStream, Option<std::os::unix::net::SocketAddr>);

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        #[cfg(target_os = "windows")]
        return Poll::Ready(futures::ready!(self.inner.poll_accept(cx))
            .map(|(inner, addr)|(UnixStream { inner }, Some(addr))));

        #[cfg(not(target_os = "windows"))] {
            match futures::ready!(Pin::new(&mut self.get_mut().incoming()).poll_accept(cx)) {
                Ok(inner) => {
                    let addr = inner.peer_addr().ok();
                    Poll::Ready(Some((UnixStream { inner }, addr)))
                }

                Err(_) => Poll::Ready(None)
            }
        }
    }
}

#[cfg(target_os = "windows")]
#[pin_project]
#[derive(Debug)]
pub struct UnixStream {
    #[pin]
    inner: crate::windows::UnixStream
}

#[cfg(not(target_os = "windows"))]
#[pin_project]
#[derive(Debug)]
pub struct UnixStream {
    #[pin]
    inner: tokio::net::UnixStream
}

impl UnixStream {
    #[cfg(target_os = "windows")]
    pub async fn connect<P: AsRef<Path>>(bind_path: P) -> std::io::Result<Self> {
        crate::stream::UnixStream::connect(bind_path)?.await.map(|inner| Self { inner })
    }

    #[cfg(not(target_os = "windows"))]
    pub async fn connect<P: AsRef<Path>>(bind_path: P) -> std::io::Result<Self> {
        tokio::net::UnixStream::connect(bind_path).await.map(|inner| Self { inner })
    }
}

impl AsyncRead for UnixStream {
    fn poll_read(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut [u8]) -> Poll<std::io::Result<usize>> {
        self.project().inner.poll_read(cx, buf)
    }
}

impl AsyncWrite for UnixStream {
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize, Error>> {
        self.project().inner.poll_write(cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        self.project().inner.poll_flush(cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        self.project().inner.poll_shutdown(cx)
    }
}