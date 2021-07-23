use anyhow::Result;
use serde::Serialize;
use std::io::Read;
use wasmer::*;
use wasmer_wasi::{Fd, Pipe, WasiFs, WasiState, ALL_RIGHTS, VIRTUAL_ROOT_FD};

mod virtual_file;
use virtual_file::VirtualFile;

#[derive(Clone, Serialize)]
#[allow(non_snake_case)]
struct KatexOptions {
	displayMode: bool,
	throwOnError: bool,
}

impl Default for KatexOptions {
	fn default() -> Self {
		KatexOptions {
			displayMode: false,
			throwOnError: true,
		}
	}
}

fn main() -> Result<()> {
	let store = Store::default();
	let module = Module::new(&store, include_bytes!("../vendor/qjs.wasm"))?;

	let equation = r"c = \pm\sqrt{a^2 + b^2}";
	let options = KatexOptions {
		displayMode: true,
		..KatexOptions::default()
	};

	// let mut wasi_fs = WasiFs::open_file_at(&mut self, base, file, open_flags, name, rights, rights_inheriting, flags)
	let mut wasi_env = WasiState::new("qjs")
		.setup_fs(Box::new(move |fs: &mut WasiFs| {
			let katex = include_bytes!("../vendor/katex.min.js");
			let mut source_vec = Vec::with_capacity(katex.len());
			source_vec.extend_from_slice(katex);

			// newer versions of QuickJS support a -I flag to include a script file before evaluating
			// but the pre-compiled WASM/WASI module is old, so we append to our virtual file
			source_vec.extend_from_slice(
				format!(
					"console.log(katex.renderToString({}, {}))",
					serde_json::to_string(equation).unwrap(),
					serde_json::to_string(&options).unwrap()
				)
				.as_bytes(),
			);

			fs.open_file_at(
				VIRTUAL_ROOT_FD,
				Box::new(VirtualFile::new(source_vec)),
				Fd::READ,
				"script.js".to_owned(),
				ALL_RIGHTS,
				ALL_RIGHTS,
				0,
			)
			.unwrap();
			Ok(())
		}))
		.arg("--script")
		.arg("/script.js")
		.stdout(Box::new(Pipe::new()))
		.finalize()?;
	let import_object = wasi_env.import_object(&module)?;
	let instance = Instance::new(&module, &import_object)?;

	let start = instance.exports.get_function("_start")?;
	start.call(&[])?;

	let mut state = wasi_env.state();
	let wasi_stdout = state.fs.stdout_mut()?.as_mut().unwrap();
	let mut buf = String::new();
	wasi_stdout.read_to_string(&mut buf)?;
	println!("{}", buf.trim());

	Ok(())
}
