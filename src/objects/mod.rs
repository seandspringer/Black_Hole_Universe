//! mod.rs
//!
//! For exposing the modules in the objects folder to
//! eachother and to parent modules

pub mod button;
pub mod clocks;
pub mod gamestate;
pub mod gauss;
pub mod movables;
pub mod plugins;
pub mod sliders;
pub mod traits;

pub use self::plugins::BlackHoleUniverse;
