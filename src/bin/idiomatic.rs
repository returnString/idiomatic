use idiomatic::codegen_rust::server::RustServer;
use idiomatic::{CodeGenerator, Config, Error, Result, Service};
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
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

	let mut out_writer = BufWriter::new(File::create(src_dir.join(codegen.source_file()))?);
	codegen.config(&config, &mut out_writer)?;

	for service in &services {
		codegen.service(service, &mut out_writer)?;
	}
	out_writer.flush()?;

	for (program, args) in codegen.post_commands() {
		let status = Command::new(program)
			.current_dir(&codegen_proj_dir)
			.args(args)
			.status()?;

		if !status.success() {
			return Err(Error::CommandError);
		}
	}

	Ok(())
}
