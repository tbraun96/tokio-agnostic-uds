use crate::windows::{UnixListener, UnixStream};

use futures::Stream;
use futures::task::{Poll, Context};
use std::pin::Pin;
use mio_uds_windows::net::SocketAddr;

/// Stream of listeners
#[derive(Debug)]
pub struct Incoming {
    inner: UnixListener,
}

impl Incoming {
    pub(crate) fn new(listener: UnixListener) -> Incoming {
        Incoming { inner: listener }
    }
}

impl Stream for Incoming {
    type Item = (UnixStream, SocketAddr);

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.inner.poll_accept(cx)
    }
}

