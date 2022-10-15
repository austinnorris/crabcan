use nix::unistd::sethostname;
use rand::seq::SliceRandom;

use crate::errors::ErrCode;

const HOSTNAME_ADJ: [&'static str; 12] = [
    "tiny", "small", "normal", "medium", "large", "huge", "silent", "noisy", "rusty", "spotted",
    "crooked", "round",
];

const HOSTNAME_COLORS: [&'static str; 12] = [
    "red", "blue", "green", "brown", "purple", "yellow", "orange", "gold", "pink", "white",
    "black", "gray",
];

const HOSTNAME_OBJECT: [&'static str; 12] = [
    "piano",
    "drum",
    "guitar",
    "synth",
    "bass",
    "oboe",
    "clarinet",
    "violin",
    "saxophone",
    "trumpet",
    "cello",
    "flute",
];

pub fn generate_hostname() -> Result<String, ErrCode> {
    let mut rng = rand::thread_rng();
    Ok(format!(
        "{}-{}-{}",
        HOSTNAME_ADJ.choose(&mut rng).ok_or(ErrCode::RngError)?,
        HOSTNAME_COLORS.choose(&mut rng).ok_or(ErrCode::RngError)?,
        HOSTNAME_OBJECT.choose(&mut rng).ok_or(ErrCode::RngError)?
    ))
}

pub fn set_container_hostname(hostname: &str) -> Result<(), ErrCode> {
    match sethostname(hostname) {
        Ok(_) => {
            log::debug!("Container hostname set to {}", hostname);
            Ok(())
        }
        Err(_) => {
            log::error!("Cannot set hostname {} for container", hostname);
            Err(ErrCode::HostnameError(0))
        }
    }
}
