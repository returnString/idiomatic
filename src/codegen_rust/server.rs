use crate::codegen::CodeGenerator;
use crate::{Config, Result, Service, Type};
use convert_case::{Case, Casing};
use std::io::Write;

pub struct RustServer;

fn type_name(s: &str) -> String {
	s.to_case(Case::Pascal)
}

fn rust_type(t: &Type) -> String {
	match t {
		Type::String => "String".into(),
	}
}

impl CodeGenerator for RustServer {
	fn name(&self) -> &str {
		"rust_server"
	}

	fn source_dir(&self) -> &str {
		"src"
	}

	fn source_file(&self) -> &str {
		"lib.rs"
	}

	fn project_files(&self, config: &Config) -> Vec<(String, String)> {
		vec![(
			"Cargo.toml".into(),
			format!(include_str!("Cargo.toml"), project_name = config.project_name),
		)]
	}

	fn config(&self, config: &Config, w: &mut impl Write) -> Result<()> {
		for principal in &config.principals {
			write!(w, "pub struct {}Principal {{", type_name(&principal.id))?;
			for (name, ty) in &principal.attributes {
				write!(w, "pub {}: {},", name, rust_type(ty))?;
			}
			write!(w, "}}")?;
		}

		Ok(())
	}

	fn service(&self, service: &Service, w: &mut impl Write) -> Result<()> {
		for endpoint in &service.endpoints {
			write!(w, "pub struct {}Request {{", type_name(&endpoint.id))?;
			if let Some(req) = &endpoint.req {
				for (name, ty) in req {
					write!(w, "pub {}: {},", name, rust_type(ty))?;
				}
			}
			write!(w, "}}")?;

			write!(w, "pub struct {}Response {{", type_name(&endpoint.id))?;
			if let Some(res) = &endpoint.res {
				for (name, ty) in res {
					write!(w, "pub {}: {},", name, rust_type(ty))?;
				}
			}
			write!(w, "}}")?;
		}

		write!(w, "pub trait {}Service {{", type_name(&service.id))?;
		for endpoint in &service.endpoints {
			let endpoint_type_prefix = type_name(&endpoint.id);
			write!(w, "fn {}(req: {}Request", endpoint.id, endpoint_type_prefix)?;
			if let Some(principal) = &endpoint.principal {
				write!(w, ", caller: {}Principal", type_name(principal))?;
			}
			write!(w, ") -> {}Response;", endpoint_type_prefix)?;
		}
		write!(w, "}}")?;
		Ok(())
	}
}
