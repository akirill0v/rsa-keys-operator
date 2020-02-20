#![warn(rust_2018_idioms)]
#![allow(unused_imports)]

#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;
#[macro_use]
extern crate prometheus;

pub type Result<T> = std::result::Result<T, anyhow::Error>;

pub mod mounter;
pub mod rsa_generator;
pub mod secret;
pub mod settings;
/// State machinery for kube, as exposeable to actix
pub mod state;
pub mod store;
pub mod utils;

pub use settings::Settings;
pub use state::Controller;
pub use store::Store;
