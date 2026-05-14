use alloc::vec::Vec;
use alloc::boxed::Box;
use super::vfs::{FileSystem, File, Result, Error};

pub struct RamDisk {
    data: Vec<u8>,
}

impl RamDisk {
    pub fn new(size: usize) -> Self {
        RamDisk {
            data: alloc::vec![0; size],
        }
    }
}

pub struct Jfs {
    storage: RamDisk,
    write_ptr: usize,
}

impl Jfs {
    pub fn new(size: usize) -> Self {
        Jfs {
            storage: RamDisk::new(size),
            write_ptr: 0,
        }
    }
}

impl FileSystem for Jfs {
    fn open<'a>(&'a mut self, _path: &str) -> Result<Box<dyn File + 'a>> {
        // In an append-only log, we might just have one 'file'
        // or a simple stream.
        Err(Error::NotFound)
    }

    fn create<'a>(&'a mut self, _path: &str) -> Result<Box<dyn File + 'a>> {
        // Return a handler that appends to the log
        Ok(Box::new(JfsFile {
            jfs: self,
        }))
    }

    fn mkdir(&mut self, _path: &str) -> Result<()> {
        Err(Error::Other)
    }

    fn exists(&self, _path: &str) -> bool {
        false
    }
}

pub struct JfsFile<'a> {
    jfs: &'a mut Jfs,
}

impl<'a> File for JfsFile<'a> {
    fn read(&mut self, _buf: &mut [u8]) -> Result<usize> {
        Ok(0)
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let len = buf.len();
        if self.jfs.write_ptr + len > self.jfs.storage.data.len() {
            return Err(Error::DiskFull);
        }
        
        self.jfs.storage.data[self.jfs.write_ptr..self.jfs.write_ptr + len]
            .copy_from_slice(buf);
        self.jfs.write_ptr += len;
        
        Ok(len)
    }

    fn seek(&mut self, _pos: u64) -> Result<u64> {
        Ok(0)
    }

    fn size(&self) -> u64 {
        self.jfs.write_ptr as u64
    }
}
