use crate::{Error, Point};

#[cfg(target_os = "linux")]
mod gnome;
#[cfg(target_os = "linux")]
mod hyprland;
#[cfg(target_os = "linux")]
mod kwin;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "linux")]
pub fn get_position() -> Result<Point, Error> {
    linux::get_position()
}

#[cfg(target_os = "windows")]
pub fn get_position() -> Result<Point, Error> {
    windows::get_position()
}

#[cfg(target_os = "macos")]
pub fn get_position() -> Result<Point, Error> {
    macos::get_position()
}

#[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
pub fn get_position() -> Result<Point, Error> {
    Err(Error::Unsupported("unsupported OS".into()))
}
