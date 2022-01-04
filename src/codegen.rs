use crate::{Config, Result, Service};
use std::io::Write;

pub trait CodeGenerator {
	fn name(&self) -> &str;
	fn source_dir(&self) -> &str;
	fn source_file(&self) -> &str;

	fn project_files(&self, _config: &Config) -> Vec<(String, String)> {
		vec![]
	}

	fn config(&self, service: &Config, w: &mut impl Write) -> Result<()>;
	fn service(&self, service: &Service, w: &mut impl Write) -> Result<()>;
}
