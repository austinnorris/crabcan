use nix::sys::socket::{recv, send, socketpair, AddressFamily, MsgFlags, SockFlag, SockType};
use std::os::unix::io::RawFd;

use crate::errors::ErrCode;

pub fn generate_socket_pair() -> Result<(RawFd, RawFd), ErrCode> {
    match socketpair(
        AddressFamily::Unix,
        SockType::SeqPacket,
        None,
        SockFlag::SOCK_CLOEXEC,
    ) {
        Ok(res) => Ok(res),
        Err(_) => Err(ErrCode::SocketError(0)),
    }
}
