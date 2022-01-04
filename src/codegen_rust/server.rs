use crate::codegen::CodeGenerator;
use crate::{Config, Endpoint, Result, Service, Type};
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

fn http_scope_fn(s: &Service) -> String {
	format!("{}_http_scope", s.id)
}

fn http_endpoint_fn(s: &Service, e: &Endpoint) -> String {
	format!("{}_http_handler_{}", s.id, e.id)
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
			format!(include_str!("templates/Cargo.toml"), project_name = config.project_name),
		)]
	}

	fn post_commands(&self) -> Vec<(String, Vec<String>)> {
		vec![
			("cargo".into(), vec!["fmt".into()]),
			("cargo".into(), vec!["check".into()]),
		]
	}

	fn config(&self, config: &Config, w: &mut impl Write) -> Result<()> {
		write!(w, "#[async_trait::async_trait] pub trait HttpPrincipalResolver<P> {{")?;
		write!(
			w,
			"async fn resolve(&self, req: &actix_web::HttpRequest) -> Result<P, actix_web::Error>;"
		)?;
		write!(w, "}}")?;

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
			write!(
				w,
				"#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)] pub struct {}Request {{",
				type_name(&endpoint.id)
			)?;
			if let Some(req) = &endpoint.req {
				for (name, ty) in req {
					write!(w, "pub {}: {},", name, rust_type(ty))?;
				}
			}
			write!(w, "}}")?;

			write!(
				w,
				"#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)] pub struct {}Response {{",
				type_name(&endpoint.id)
			)?;
			if let Some(res) = &endpoint.res {
				for (name, ty) in res {
					write!(w, "pub {}: {},", name, rust_type(ty))?;
				}
			}
			write!(w, "}}")?;
		}

		write!(
			w,
			"#[async_trait::async_trait] pub trait {}Service {{",
			type_name(&service.id)
		)?;
		for endpoint in &service.endpoints {
			let endpoint_type_prefix = type_name(&endpoint.id);
			write!(
				w,
				"async fn {}(&self, req: &{}Request",
				endpoint.id, endpoint_type_prefix
			)?;
			if let Some(principal) = &endpoint.principal {
				write!(w, ", caller: &{}Principal", type_name(principal))?;
			}
			write!(w, ") -> {}Response;", endpoint_type_prefix)?;
		}
		write!(w, "}}")?;

		for endpoint in &service.endpoints {
			write!(
				w,
				"async fn {}(svc: actix_web::web::Data<dyn {}Service>, req: actix_web::web::Json<{}Request>",
				http_endpoint_fn(service, endpoint),
				type_name(&service.id),
				type_name(&endpoint.id),
			)?;

			if let Some(principal) = &endpoint.principal {
				write!(
					w,
					", resolver: actix_web::web::Data<dyn HttpPrincipalResolver<{}Principal>>, http_req: actix_web::HttpRequest", type_name(principal),
				)?;
			}

			write!(w, ") -> impl actix_web::Responder {{")?;
			write!(w, "actix_web::HttpResponse::Ok().json(svc.{}(&req", endpoint.id)?;
			if endpoint.principal.is_some() {
				write!(w, ", match resolver.resolve(&http_req).await {{ Ok(ref p) => p, Err(err) => return actix_web::HttpResponse::Unauthorized().finish() }}")?;
			}
			write!(w, ").await)")?;
			write!(w, "}}")?;
		}

		write!(
			w,
			"pub fn {}(svc: std::sync::Arc<dyn {}Service>) -> actix_web::Scope {{",
			http_scope_fn(service),
			type_name(&service.id),
		)?;
		write!(w, "actix_web::web::scope(\"{}\")", service.id)?;
		write!(w, ".app_data(actix_web::web::Data::from(svc))")?;
		for endpoint in &service.endpoints {
			write!(
				w,
				".service(actix_web::web::resource(\"{}\").route(actix_web::web::post().to({})))",
				endpoint.id,
				http_endpoint_fn(service, endpoint)
			)?;
		}
		write!(w, "}}")?;

		Ok(())
	}
}
