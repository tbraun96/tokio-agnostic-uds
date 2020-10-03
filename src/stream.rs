use futures::Future;
use iovec::IoVec;
use mio::Ready;
use mio_uds_windows as mio_uds;
use mio_uds_windows::net::{self, SocketAddr};

use std::fmt;
use std::io;
use std::net::Shutdown;
use std::os::windows::io::{AsRawSocket, RawSocket};
use std::path::Path;
use tokio::io::{PollEvented, AsyncWrite, AsyncRead};
use futures::task::{Poll, Context};
use tokio::macros::support::Pin;
use futures::io::Error;

/// A structure representing a connected Unix socket.
///
/// This socket can be connected directly with `UnixStream::connect` or accepted
/// from a listener with `UnixListener::incoming`. Additionally, a pair of
/// anonymous Unix sockets can be created with `UnixStream::pair`.
pub struct UnixStream {
    io: PollEvented<mio_uds::UnixStream>,
}

/// Future returned by `UnixStream::connect` which will resolve to a
/// `UnixStream` when the stream is connected.
#[derive(Debug)]
pub struct ConnectFuture {
    inner: State,
}

#[derive(Debug)]
enum State {
    Waiting(UnixStream),
    Error(io::Error),
    Empty,
}

impl UnixStream {
    /// Connects to the socket named by `path`.
    ///
    /// This function will create a new Unix socket and connect to the path
    /// specified, associating the returned stream with the default event loop's
    /// handle.
    pub fn connect<P>(path: P) -> io::Result<ConnectFuture>
    where
        P: AsRef<Path>,
    {
        let res = mio_uds::UnixStream::connect(path)
            .map(UnixStream::new);

        let inner = match res {
            Ok(stream) => State::Waiting(stream?),
            Err(e) => State::Error(e),
        };

        Ok(ConnectFuture { inner })
    }

    /// Consumes a `UnixStream` in the standard library and returns a
    /// nonblocking `UnixStream` from this crate.
    ///
    /// The returned stream will be associated with the given event loop
    /// specified by `handle` and is ready to perform I/O.
    pub fn from_std(stream: net::UnixStream) -> io::Result<UnixStream> {
        let stream = mio_uds::UnixStream::from_stream(stream)?;
        let io = PollEvented::new(stream)?;
        Ok(UnixStream { io })
    }

    pub(crate) fn new(stream: mio_uds::UnixStream) -> io::Result<UnixStream> {
        let io = PollEvented::new(stream)?;
        Ok(UnixStream { io })
    }

    /// Test whether this socket is ready to be read or not.
    pub fn poll_read_ready(&self, cx: &mut Context<'_>, ready: Ready) -> Poll<Result<Ready, io::Error>> {
        self.io.poll_read_ready(cx,ready)
    }

    /// Test whether this socket is ready to be written to or not.
    pub fn poll_write_ready(&self, cx: &mut Context<'_>) -> Poll<Result<Ready, io::Error>> {
        self.io.poll_write_ready(cx)
    }

    /// Returns the socket address of the local half of this connection.
    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.io.get_ref().local_addr()
    }

    /// Returns the socket address of the remote half of this connection.
    pub fn peer_addr(&self) -> io::Result<SocketAddr> {
        self.io.get_ref().peer_addr()
    }

    /// Returns the value of the `SO_ERROR` option.
    pub fn take_error(&self) -> io::Result<Option<io::Error>> {
        self.io.get_ref().take_error()
    }

    /// Shuts down the read, write, or both halves of this connection.
    ///
    /// This function will cause all pending and future I/O calls on the
    /// specified portions to immediately return with an appropriate value
    /// (see the documentation of `Shutdown`).
    pub fn shutdown(&self, how: Shutdown) -> io::Result<()> {
        self.io.get_ref().shutdown(how)
    }
}

impl AsyncRead for UnixStream {
    fn poll_read(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut [u8]) -> Poll<io::Result<usize>> {
        <&UnixStream>::poll_read(Pin::new(&mut &*self), cx, buf)
    }
}

impl AsyncWrite for UnixStream {
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<io::Result<usize>> {
        <&UnixStream>::poll_write(Pin::new(&mut &*self), cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        <&UnixStream>::poll_flush(Pin::new(&mut &*self), cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        <&UnixStream>::poll_shutdown(Pin::new(&mut &*self), cx)
    }
}

impl<'a> AsyncRead for &'a UnixStream {
    fn poll_read(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut [u8]) -> Poll<io::Result<usize>> {
        if let Poll::Pending = <UnixStream>::poll_read_ready(*self, cx,Ready::readable())? {
            return Poll::Pending;
        }
        let r =  {
            // The `IoVec` type can't have a 0-length size, so we create a bunch
            // of dummy versions on the stack with 1 length which we'll quickly
            // overwrite.
            let b1: &mut [u8] = &mut [0];
            let b2: &mut [u8] = &mut [0];
            let b3: &mut [u8] = &mut [0];
            let b4: &mut [u8] = &mut [0];
            let b5: &mut [u8] = &mut [0];
            let b6: &mut [u8] = &mut [0];
            let b7: &mut [u8] = &mut [0];
            let b8: &mut [u8] = &mut [0];
            let b9: &mut [u8] = &mut [0];
            let b10: &mut [u8] = &mut [0];
            let b11: &mut [u8] = &mut [0];
            let b12: &mut [u8] = &mut [0];
            let b13: &mut [u8] = &mut [0];
            let b14: &mut [u8] = &mut [0];
            let b15: &mut [u8] = &mut [0];
            let b16: &mut [u8] = &mut [0];
            let mut bufs: [&mut IoVec; 16] = [
                b1.into(), b2.into(), b3.into(), b4.into(),
                b5.into(), b6.into(), b7.into(), b8.into(),
                b9.into(), b10.into(), b11.into(), b12.into(),
                b13.into(), b14.into(), b15.into(), b16.into(),
            ];

            bufs[0] = IoVec::from_bytes_mut(buf).unwrap();
            let n = 1;
            self.io.get_ref().read_bufs(&mut bufs[..n])
        };

        match r {
            Ok(n) => {
                Poll::Ready(Ok(n))
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                self.io.clear_read_ready(cx,Ready::readable())?;
                Poll::Pending
            }
            Err(e) => Poll::Ready(Err(e)),
        }
    }
}

impl<'a> AsyncWrite for &'a UnixStream {
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<io::Result<usize>> {
        if let Poll::Pending = <UnixStream>::poll_write_ready(*self, cx)? {
            return Poll::Pending;
        }
        let r = {
            // The `IoVec` type can't have a zero-length size, so create a dummy
            // version from a 1-length slice which we'll overwrite with the
            // `bytes_vec` method.
            static DUMMY: &[u8] = &[0];
            let iovec = <&IoVec>::from(DUMMY);
            let mut bufs = [
                iovec, iovec, iovec, iovec, iovec, iovec, iovec, iovec, iovec, iovec, iovec, iovec, iovec,
                iovec, iovec, iovec,
            ];

            bufs[0] = IoVec::from_bytes(buf).unwrap();
            let n = 1;
            self.io.get_ref().write_bufs(&bufs[..n])
        };

        match r {
            Ok(n) => {
                Poll::Ready(Ok(n))
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                self.io.clear_write_ready(cx)?;
                Poll::Pending
            }
            Err(e) => Poll::Ready(Err(e)),
        }
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        Poll::Ready(self.shutdown(Shutdown::Write))
    }
}

impl fmt::Debug for UnixStream {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.io.get_ref().fmt(f)
    }
}

impl AsRawSocket for UnixStream {
    fn as_raw_socket(&self) -> RawSocket {
        self.io.get_ref().as_raw_socket()
    }
}

impl Future for ConnectFuture {
    type Output = io::Result<UnixStream>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        use std::mem;

        match self.inner {
            State::Waiting(ref mut stream) => {
                if let Poll::Pending = stream.io.poll_write_ready(cx)? {
                    return Poll::Pending
                }

                if let Some(e) = stream.io.get_ref().take_error()? {
                    return Poll::Ready(Err(e))
                }
            }
            State::Error(_) => {
                let e = match mem::replace(&mut self.inner, State::Empty) {
                    State::Error(e) => e,
                    _ => unreachable!(),
                };

                return Poll::Ready(Err(e))
            },
            State::Empty => panic!("can't poll stream twice"),
        }

        match mem::replace(&mut self.inner, State::Empty) {
            State::Waiting(stream) => Poll::Ready(Ok(stream)),
            _ => unreachable!(),
        }
    }
}
