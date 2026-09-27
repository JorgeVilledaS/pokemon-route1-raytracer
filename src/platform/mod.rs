#[cfg(windows)]
mod windows;

#[cfg(windows)]
pub use windows::run;

#[cfg(not(windows))]
pub fn run(_scene: crate::scene::Scene, _target: crate::core::Vec3) -> std::io::Result<()> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "the interactive std-only window currently targets Windows",
    ))
}
