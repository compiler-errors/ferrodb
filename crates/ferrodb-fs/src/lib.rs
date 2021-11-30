#![feature(let_else)]

mod error;

use std::collections::HashMap;
use std::fs::OpenOptions;
use std::os::unix::fs::FileExt;
use std::os::unix::prelude::MetadataExt;
use std::sync::Arc;

use camino::Utf8PathBuf;
pub use error::Error;
use ferrodb_page::{allocate_page, page_size, PageHandle, PageRef};
use ferrodb_util::id_type;
use parking_lot::Mutex;

id_type!(pub FileId);

pub type PageIndex = usize;
type FileHandle = Arc<Mutex<FileInner>>;

pub struct FileManager {
    ids: Mutex<HashMap<String, FileId>>,
    paths: Mutex<HashMap<FileId, Utf8PathBuf>>,
    files: Mutex<HashMap<(FileId, PageIndex), FileHandle>>,
}

impl FileManager {
    pub fn id(&self, name: &str) -> FileId {
        let mut ids = self.ids.lock();

        if let Some(id) = ids.get(name) {
            *id
        } else {
            let id = FileId::new();
            ids.insert(name.to_owned(), id);

            let mut path = Utf8PathBuf::from(".");
            path.set_file_name(name);
            self.paths.lock().insert(id, path);

            id
        }
    }

    pub fn clean(&self, file: FileId, page: PageIndex) -> Result<FileRef, Error> {
        let inner = self.files.lock().entry((file, page)).or_default().clone();
        let mut inner = inner.lock();

        if let Some(handle) = &inner.clean {
            if let Ok(page_ref) = handle.pin() {
                return Ok(FileRef(page_ref, false));
            }
        }

        let (handle, page_ref) = self.read_to_page(file, page)?;
        inner.clean = Some(handle);

        Ok(FileRef(page_ref, false))
    }

    pub fn dirty(&self, file: FileId, page: PageIndex) -> Result<FileRef, Error> {
        let inner = self.files.lock().entry((file, page)).or_default().clone();
        let mut inner = inner.lock();

        if let Some((_, page_ref)) = &inner.dirty {
            return Ok(FileRef(page_ref.clone(), true));
        }

        // TODO(michael): We can also read from clean to avoid fs, if it exists and is
        // pinnable.
        let (handle, page_ref) = self.read_to_page(file, page)?;
        inner.dirty = Some((handle, page_ref.clone()));
        Ok(FileRef(page_ref, true))
    }

    fn read_to_page(&self, file: FileId, page: PageIndex) -> Result<(PageHandle, PageRef), Error> {
        let (page_handle, page_ref) = allocate_page()?;
        let mut buf = page_ref.write();

        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(&self.paths.lock()[&file])?;
        let size = file.metadata()?.size(); // TODO(mgoulet): I guess we just don't support 32-bit.
        let offset = (page * page_size()) as u64;

        if offset < size {
            let bytes_to_read = ((size - offset) as usize).min(page_size());
            file.read_exact_at(&mut buf[..bytes_to_read], offset)?;
            buf[bytes_to_read..].fill(0);
        } else {
            buf.fill(0);
        }

        drop(buf);
        Ok((page_handle, page_ref))
    }

    pub fn sync(&self, file: FileId, page: PageIndex) -> Result<(), Error> {
        let Some(inner) = self.files.lock().get(&(file, page)).cloned()
            else { return Ok(()); };
        let mut inner = inner.lock();

        // Don't drop handle immediately, so we name it `_handle` instead of `handle`
        if let Some((_handle, page_ref)) = inner.dirty.take() {
            let mut buf = page_ref.write();

            let file = OpenOptions::new()
                .write(true)
                .create(true)
                .open(&self.paths.lock()[&file])?;

            let offset = (page * page_size()) as u64;

            file.write_all_at(&mut buf, offset)?;

            // Overwrite the clean buffer if we have one.
            if let Some(clean) = &inner.clean {
                if let Ok(page_ref) = clean.pin() {
                    page_ref.write().copy_from_slice(&*buf);
                } else {
                    // Clear the clean buffer if it's invalidated.
                    inner.clean = None;
                }
            }
        }

        Ok(())
    }
}

#[derive(Default)]
pub struct FileInner {
    clean: Option<PageHandle>,
    dirty: Option<(PageHandle, PageRef)>,
}

pub struct FileRef(PageRef, bool /* writeable */);
