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

pub fn send_boolean(fd: RawFd, msg: bool) -> Result<(), ErrCode> {
    let data: [u8; 1] = [msg.into()];
    if let Err(e) = send(fd, &data, MsgFlags::empty()) {
        log::error!("Cannot send boolean through socket: {:?}", e);
        return Err(ErrCode::SocketError(1));
    };
    Ok(())
}

pub fn recv_boolean(fd: RawFd) -> Result<bool, ErrCode> {
    let mut data: [u8; 1] = [0];
    if let Err(e) = recv(fd, &mut data, MsgFlags::empty()) {
        log::error!("Cannot receive boolean from socket: {:?}", e);
        return Err(ErrCode::SocketError(2));
    }
    Ok(data[0] == 1)
}
