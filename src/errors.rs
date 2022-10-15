use std::fmt;
use std::process::exit;

#[derive(Debug)]
pub enum ErrCode {
    InvalidArgument(&'static str),
    NotSupported(u8),
    ContainerError(u8),
    SocketError(u8),
    ChildProcessError(u8),
    HostnameError(u8),
    RngError,
}

impl ErrCode {
    pub fn get_retcode(&self) -> i32 {
        1
    }
}

impl fmt::Display for ErrCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            Self::InvalidArgument(element) => write!(f, "InvalidArgument: {}", element),
            _ => write!(f, "{:?}", self),
        }
    }
}

pub fn exit_with_return_code(res: Result<(), ErrCode>) {
    match res {
        Ok(_) => {
            log::debug!("Exiting without error");
            exit(0);
        }
        Err(e) => {
            let retcode = e.get_retcode();
            log::error!("Error on exit:\n\t{}", e);
            exit(retcode);
        }
    }
}
