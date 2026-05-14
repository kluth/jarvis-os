use alloc::boxed::Box;

pub enum Error {
    NotFound,
    AlreadyExists,
    PermissionDenied,
    IsDirectory,
    NotADirectory,
    DiskFull,
    InvalidPath,
    Other,
}

pub type Result<T> = core::result::Result<T, Error>;

pub trait FileSystem {
    fn open<'a>(&'a mut self, path: &str) -> Result<Box<dyn File + 'a>>;
    fn create<'a>(&'a mut self, path: &str) -> Result<Box<dyn File + 'a>>;
    fn mkdir(&mut self, path: &str) -> Result<()>;
    fn exists(&self, path: &str) -> bool;
}

pub trait File {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize>;
    fn write(&mut self, buf: &[u8]) -> Result<usize>;
    fn seek(&mut self, pos: u64) -> Result<u64>;
    fn size(&self) -> u64;
}

// Global VFS Root
pub struct Vfs {
    // For now a simple single mount point
    pub root: Option<Box<dyn FileSystem>>,
}

impl Vfs {
    pub const fn new() -> Self {
        Vfs { root: None }
    }
}
