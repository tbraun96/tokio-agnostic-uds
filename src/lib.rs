#![doc(html_root_url = "https://docs.rs/tokio-uds/0.2.3")]
#![deny(missing_debug_implementations)]

//! Unix Domain Sockets for Tokio.
//!
//! This crate provides APIs for using Unix Domain Sockets with Tokio.
#[cfg(target_os = "windows")]
mod incoming;
#[cfg(target_os = "windows")]
mod listener;
#[cfg(target_os = "windows")]
mod stream;
mod merge;

#[cfg(target_os = "windows")]
pub(crate) mod windows {
    pub use crate::incoming::Incoming;
    pub use crate::listener::UnixListener;
    pub use crate::stream::{UnixStream, ConnectFuture};
}

pub use merge::{UnixStream, UnixListener};