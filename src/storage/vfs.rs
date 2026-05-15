use alloc::boxed::Box;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    NotFound,
    DiskFull,
    InvalidPath,
    Other,
}

pub trait FileSystem {
    fn open<'a>(&'a mut self, path: &str) -> Result<Box<dyn File + 'a>>;
    fn create<'a>(&'a mut self, path: &str) -> Result<Box<dyn File + 'a>>;
    fn mkdir(&mut self, path: &str) -> Result<()>;
}

pub trait File {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize>;
    fn write(&mut self, buf: &[u8]) -> Result<usize>;
    fn seek(&mut self, pos: usize) -> Result<()>;
}

// Global VFS Root
#[derive(Default)]
pub struct Vfs {
    root: Option<Box<dyn FileSystem>>,
}

impl Vfs {
    pub const fn new() -> Self {
        Vfs { root: None }
    }

    pub fn mount(&mut self, fs: Box<dyn FileSystem>) {
        self.root = Some(fs);
    }

    pub fn get_root(&mut self) -> Option<&mut (dyn FileSystem + 'static)> {
        self.root.as_deref_mut()
    }
}
