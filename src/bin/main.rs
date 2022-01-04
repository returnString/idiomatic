use idiomatic::codegen_rust::server::RustServer;
use idiomatic::{CodeGenerator, Config, Result, Service};
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use structopt::StructOpt;

fn load_services(dir: &Path) -> Result<Vec<Service>> {
	let mut services = vec![];

	for entry in fs::read_dir(dir.join("services"))?.filter_map(|e| e.ok()) {
		let service: Service = serde_yaml::from_reader(File::open(entry.path())?)?;
		services.push(service);
	}

	Ok(services)
}

#[derive(Debug, StructOpt)]
#[structopt(name = "idiomatic", about = "API code generator")]
struct Opt {
	src: PathBuf,
	out: PathBuf,
}

fn main() -> Result<()> {
	let opt = Opt::from_args();

	let config: Config = serde_yaml::from_reader(File::open(opt.src.join("config.yml"))?)?;

	let services = load_services(&opt.src)?;
	let codegen = RustServer;

	let codegen_proj_dir = opt.out.join(codegen.name());
	let src_dir = codegen_proj_dir.join(codegen.source_dir());

	fs::create_dir_all(&src_dir)?;

	for (name, contents) in codegen.project_files(&config) {
		fs::write(codegen_proj_dir.join(name), contents)?;
	}

	let mut out_file = File::create(src_dir.join(codegen.source_file()))?;
	codegen.config(&config, &mut out_file)?;

	for service in &services {
		codegen.service(service, &mut out_file)?;
	}

	Ok(())
}
