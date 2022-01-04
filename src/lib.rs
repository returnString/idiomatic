pub mod codegen;
pub mod service;
pub use codegen::*;
pub use service::*;

pub mod codegen_rust;

#[derive(thiserror::Error, Debug)]
pub enum Error {
	#[error("io error: {0}")]
	Io(#[from] std::io::Error),
	#[error("yaml error: {0}")]
	Yaml(#[from] serde_yaml::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
