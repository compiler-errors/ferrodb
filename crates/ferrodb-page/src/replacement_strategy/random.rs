use parking_lot::Mutex;
use rand::{thread_rng, Rng};

use super::{NoPages, ReplacementStrategy};
use crate::PageId;

pub struct RandomReplacementStrategy {
    pages: Mutex<Vec<PageId>>,
}

impl ReplacementStrategy for RandomReplacementStrategy {
    fn new(_limit: usize) -> RandomReplacementStrategy {
        RandomReplacementStrategy {
            pages: Mutex::default(),
        }
    }

    fn allocate(&self, id: PageId) {
        // NOTE(mgoulet): We could check if pages contains id, but why?
        self.pages.lock().push(id);
    }

    fn read(&self, _id: PageId) {
        // Do nothing, we don't care if a page is read from
    }

    fn write(&self, _id: PageId) {
        // Do nothing, we don't care if a page is written to
    }

    fn evict<F>(&self, mut can_evict: F) -> Result<PageId, NoPages>
    where
        F: FnMut(PageId) -> bool,
    {
        let mut pages = self.pages.lock();
        let mut skipped = 0;
        let mut rest = &mut **pages;
        let mut rng = thread_rng();

        while !rest.is_empty() {
            let candidate_idx = rng.gen_range(0..rest.len());
            let candidate = rest[candidate_idx];

            if can_evict(candidate) {
                pages.remove(skipped + candidate_idx);
                return Ok(candidate);
            } else {
                skipped += 1;
                rest.swap(0, candidate_idx);
                rest = &mut rest[1..];
            }
        }

        Err(NoPages)
    }
}
