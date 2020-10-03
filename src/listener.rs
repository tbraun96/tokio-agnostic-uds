use crate::windows::{Incoming, UnixStream};

use mio::Ready;
use mio_uds_windows as mio_uds;
use mio_uds_windows::net::{self, SocketAddr};
use tokio::io::PollEvented;
use std::fmt;
use std::io;
use std::path::Path;
use futures::task::{Poll, Context};

/// A Unix socket which can accept connections from other Unix sockets.
pub struct UnixListener {
    io: PollEvented<mio_uds::UnixListener>,
}

impl UnixListener {
    /// Creates a new `UnixListener` bound to the specified path.
    pub fn bind<P>(path: P) -> io::Result<UnixListener>
    where
        P: AsRef<Path>,
    {
        let listener = mio_uds::UnixListener::bind(path)?;
        let io = PollEvented::new(listener)?;
        Ok(UnixListener { io })
    }

    /// Consumes a `UnixListener` in the standard library and returns a
    /// nonblocking `UnixListener` from this crate.
    ///
    /// The returned listener will be associated with the given event loop
    /// specified by `handle` and is ready to perform I/O.
    #[allow(dead_code)]
    pub fn from_std(listener: net::UnixListener) -> io::Result<UnixListener> {
        let listener = mio_uds::UnixListener::from_listener(listener)?;
        let io = PollEvented::new(listener)?;
        Ok(UnixListener { io })
    }

    /// Returns the local socket address of this listener.
    #[allow(dead_code)]
    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.io.get_ref().local_addr()
    }

    /// Returns the value of the `SO_ERROR` option.
    #[allow(dead_code)]
    pub fn take_error(&self) -> io::Result<Option<io::Error>> {
        self.io.get_ref().take_error()
    }

    /// Attempt to accept a connection and create a new connected `UnixStream`
    /// if successful.
    ///
    /// This function will attempt an accept operation, but will not block
    /// waiting for it to complete. If the operation would block then a "would
    /// block" error is returned. Additionally, if this method would block, it
    /// registers the current task to receive a notification when it would
    /// otherwise not block.
    ///
    /// Note that typically for simple usage it's easier to treat incoming
    /// connections as a `Stream` of `UnixStream`s with the `incoming` method
    /// below.
    ///
    /// # Panics
    ///
    /// This function will panic if it is called outside the context of a
    /// future's task. It's recommended to only call this from the
    /// implementation of a `Future::poll`, if necessary.
    pub fn poll_accept(&self, cx: &mut Context<'_>) -> Poll<Option<(UnixStream, SocketAddr)>> {
        match futures::ready!(self.poll_accept_std(cx)) {
            Some((io, addr)) => {
                if let Ok(io) = mio_uds::UnixStream::from_stream(io) {
                    if let Ok(io) = UnixStream::new(io) {
                        Poll::Ready(Some((io, addr)))
                    } else {
                        Poll::Ready(None)
                    }
                } else {
                    Poll::Ready(None)
                }
            }

            None => {
                Poll::Pending
            }
        }
    }

    /// Attempt to accept a connection and create a new connected `UnixStream`
    /// if successful.
    ///
    /// This function is the same as `poll_accept` above except that it returns a
    /// `mio_uds::UnixStream` instead of a `tokio_udp::UnixStream`. This in turn
    /// can then allow for the stream to be associated with a different reactor
    /// than the one this `UnixListener` is associated with.
    ///
    /// This function will attempt an accept operation, but will not block
    /// waiting for it to complete. If the operation would block then a "would
    /// block" error is returned. Additionally, if this method would block, it
    /// registers the current task to receive a notification when it would
    /// otherwise not block.
    ///
    /// Note that typically for simple usage it's easier to treat incoming
    /// connections as a `Stream` of `UnixStream`s with the `incoming` method
    /// below.
    ///
    /// # Panics
    ///
    /// This function will panic if it is called outside the context of a
    /// future's task. It's recommended to only call this from the
    /// implementation of a `Future::poll`, if necessary.
    pub fn poll_accept_std(&self, cx: &mut Context<'_>) -> Poll<Option<(net::UnixStream, SocketAddr)>> {
        loop {
            let _ = futures::ready!(self.io.poll_read_ready(cx, Ready::readable()));

            match self.io.get_ref().accept_std() {
                Ok(None) => {
                    if let Err(_) = self.io.clear_read_ready(cx,Ready::readable()) {
                        return Poll::Ready(None)
                    }

                    return Poll::Pending;
                }
                Ok(Some((sock, addr))) => {
                    return Poll::Ready(Some((sock, addr)));
                }
                Err(ref err) if err.kind() == io::ErrorKind::WouldBlock => {
                    if let Err(_) = self.io.clear_read_ready(cx, Ready::readable()) {
                        return Poll::Ready(None)
                    }

                    return Poll::Pending;
                }
                Err(_) => return Poll::Ready(None),
            }
        }
    }

    /// Consumes this listener, returning a stream of the sockets this listener
    /// accepts.
    ///
    /// This method returns an implementation of the `Stream` trait which
    /// resolves to the sockets the are accepted on this listener.
    #[allow(dead_code)]
    pub fn incoming(self) -> Incoming {
        Incoming::new(self)
    }
}

impl fmt::Debug for UnixListener {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.io.get_ref().fmt(f)
    }
}