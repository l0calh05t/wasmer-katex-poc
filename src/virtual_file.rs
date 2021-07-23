use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{
	convert::TryInto,
	io::{Cursor, Read, Seek, SeekFrom, Write},
};
use wasmer_wasi::{
	types::{__wasi_filesize_t, __wasi_timestamp_t},
	WasiFile, WasiFsError,
};

#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct VirtualFile(Cursor<Vec<u8>>);

impl VirtualFile {
	pub fn new(data: Vec<u8>) -> Self {
		Self(Cursor::new(data))
	}
}

impl Seek for VirtualFile {
	fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
		self.0.seek(pos)
	}
}

impl Read for VirtualFile {
	fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
		self.0.read(buf)
	}
}

impl Write for VirtualFile {
	fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
		self.0.write(buf)
	}

	fn flush(&mut self) -> std::io::Result<()> {
		self.0.flush()
	}
}

impl Serialize for VirtualFile {
	fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		todo!();
	}
}

impl<'de> Deserialize<'de> for VirtualFile {
	fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		todo!()
	}
}

#[typetag::serde]
impl WasiFile for VirtualFile {
	fn last_accessed(&self) -> __wasi_timestamp_t {
		0
	}

	fn last_modified(&self) -> __wasi_timestamp_t {
		0
	}

	fn created_time(&self) -> __wasi_timestamp_t {
		0
	}

	fn size(&self) -> u64 {
		self.0.get_ref().len().try_into().unwrap()
	}

	fn set_len(&mut self, _new_size: __wasi_filesize_t) -> Result<(), WasiFsError> {
		todo!()
	}

	fn unlink(&mut self) -> Result<(), WasiFsError> {
		todo!()
	}

	fn bytes_available(&self) -> Result<usize, WasiFsError> {
		todo!()
	}
}
