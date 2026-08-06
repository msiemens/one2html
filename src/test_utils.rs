use onenote_parser::FileSystem;
use onenote_parser::fs::FileSource;
use std::io::Error;
use std::io::Read;
use std::sync::Arc;
use typed_path::{TypedPath, TypedPathBuf};

/// A [`FileSystem`] that only answers [`FileSystem::is_windows`].
///
/// Filename determination never touches the disk, so every other method is
/// deliberately left unimplemented — a test that starts hitting one of these is
/// exercising something it did not mean to.
#[derive(Copy, Clone)]
pub(crate) struct StubFs {
    pub(crate) windows: bool,
}

impl FileSystem for StubFs {
    fn is_directory(&self, _: TypedPath) -> Result<bool, Error> {
        unimplemented!()
    }
    fn read_dir(&self, _: TypedPath) -> Result<Vec<TypedPathBuf>, Error> {
        unimplemented!()
    }
    fn read_file(&self, _: TypedPath) -> Result<Vec<u8>, Error> {
        unimplemented!()
    }
    fn write_file(&self, _: TypedPath, _: &[u8]) -> Result<(), Error> {
        unimplemented!()
    }
    fn stream_to_file(&self, _: TypedPath, _: &mut dyn Read) -> Result<(), Error> {
        unimplemented!()
    }
    fn make_dir(&self, _: TypedPath) -> Result<(), Error> {
        unimplemented!()
    }
    fn exists(&self, _: TypedPath) -> Result<bool, Error> {
        unimplemented!()
    }
    fn open_file(&self, _: TypedPath) -> Result<Arc<dyn FileSource>, Error> {
        unimplemented!()
    }
    fn canonicalize(&self, _: TypedPath) -> Result<TypedPathBuf, Error> {
        unimplemented!()
    }
    fn is_windows(&self) -> bool {
        self.windows
    }
}
