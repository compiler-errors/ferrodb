use super::ReplacementStrategy;
use crate::{NoPages, PageId};

pub struct NoOpReplacementStrategy;

impl ReplacementStrategy for NoOpReplacementStrategy {
    fn new(_limit: usize) -> NoOpReplacementStrategy {
        NoOpReplacementStrategy
    }

    fn allocate(&self, _id: PageId) {
        // Do nothing
    }

    fn read(&self, _id: PageId) {
        // Do nothing
    }

    fn write(&self, _id: PageId) {
        // Do nothing
    }

    fn evict<F>(&self, _can_evict: F) -> Result<PageId, NoPages>
    where
        F: FnMut(PageId) -> bool,
    {
        // Do nothing, return NoPages since we don't have any pages to evict.
        Err(NoPages)
    }
}
