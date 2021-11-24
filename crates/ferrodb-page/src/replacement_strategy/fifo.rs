use std::collections::VecDeque;

use parking_lot::Mutex;

use super::ReplacementStrategy;
use crate::{NoPages, PageId};

pub struct FifoReplacementStrategy {
    pages: Mutex<VecDeque<PageId>>,
}

impl ReplacementStrategy for FifoReplacementStrategy {
    fn new(_limit: usize) -> FifoReplacementStrategy {
        todo!()
    }

    fn evict<F>(&self, mut try_evict: F) -> Result<PageId, NoPages>
    where
        F: FnMut(PageId) -> bool,
    {
        let mut pages = self.pages.lock();

        let candidate = pages
            .iter()
            .copied()
            .enumerate()
            .find(|(_, page)| try_evict(*page));

        if let Some((idx, page)) = candidate {
            pages.remove(idx);
            Ok(page)
        } else {
            Err(NoPages)
        }
    }

    fn allocate(&self, id: PageId) {
        self.pages.lock().push_back(id);
    }

    fn read(&self, id: PageId) {
        // Do nothing, we don't care about reads
    }

    fn write(&self, id: PageId) {
        // Do nothing, we don't care about writes
    }
}
