use anyhow::Result;
use serde::Serialize;
use std::io::Read;
use wasmer::*;
use wasmer_wasi::{Pipe, WasiState};

#[derive(Clone, Default, Serialize)]
struct KatexOptions {
	#[serde(rename = "displayMode", skip_serializing_if = "Option::is_none")]
	display_mode: Option<bool>,
	#[serde(rename = "throwOnError", skip_serializing_if = "Option::is_none")]
	throw_on_error: Option<bool>,
}

fn main() -> Result<()> {
	let store = Store::default();
	let module = Module::new(&store, include_bytes!("../vendor/qjs.wasm"))?;

	let equation = r"c = \pm\sqrt{a^2 + b^2}";
	let options = KatexOptions {
		display_mode: Some(true),
		..KatexOptions::default()
	};

	let katex = concat!(
		include_str!("../vendor/katex.min.js"),
		include_str!("../vendor/contrib/mhchem.min.js")
	);

	// newer versions of QuickJS support a -I flag to include a script file before evaluating
	// but the pre-compiled WASM/WASI module is old, so we concatenate everything
	let source = format!(
		"{}console.log(katex.renderToString({}, {}))",
		katex,
		serde_json::to_string(equation).unwrap(),
		serde_json::to_string(&options).unwrap(),
	);

	let mut wasi_env = WasiState::new("qjs")
		.arg("-e")
		.arg(source)
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
