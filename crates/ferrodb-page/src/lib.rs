#![feature(once_cell)]
#![feature(let_else)]

mod buffered;
mod page;
mod replacement_strategy;
mod unlimited;

use std::lazy::SyncOnceCell;

pub use self::page::{PageHandle, PageId, PageRef};
pub use self::replacement_strategy::NoPages;

static PAGE_MANAGER: SyncOnceCell<Box<dyn PageManager + Send + Sync + 'static>> =
    SyncOnceCell::new();

trait PageManager {
    fn allocate(&'static self) -> Result<(PageHandle, PageRef), NoPages>;
}

pub fn allocate_page() -> Result<(PageHandle, PageRef), NoPages> {
    PAGE_MANAGER
        .get()
        .expect("Expected Page Manager to be setup during database startup")
        .allocate()
}
