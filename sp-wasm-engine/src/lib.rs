#[macro_use]
extern crate mozjs;

pub mod error;
pub mod sandbox;

pub type Result<T> = std::result::Result<T, error::Error>;

pub mod prelude {
    pub use super::sandbox::engine::Engine;
    pub use super::sandbox::vfs::VirtualFS;
    pub use super::sandbox::Sandbox;
}
