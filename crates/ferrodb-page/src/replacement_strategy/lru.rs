use linked_hash_map::LinkedHashMap;
use parking_lot::Mutex;

use super::ReplacementStrategy;
use crate::{NoPages, PageId};

pub struct LruReplacementStrategy {
    pages: Mutex<LinkedHashMap<PageId, ()>>,
}

impl ReplacementStrategy for LruReplacementStrategy {
    fn new(_limit: usize) -> LruReplacementStrategy {
        LruReplacementStrategy {
            pages: Mutex::default(),
        }
    }

    fn evict<F>(&self, mut try_evict: F) -> Result<PageId, NoPages>
    where
        F: FnMut(PageId) -> bool,
    {
        let mut pages = self.pages.lock();

        let candidate = pages.keys().copied().find(|page| try_evict(*page));

        if let Some(page) = candidate {
            // TODO(mgoulet): Change to assert_matches when that is introduced
            pages.remove(&page).expect("Expected to remove page");
            Ok(page)
        } else {
            Err(NoPages)
        }
    }

    fn allocate(&self, page_id: PageId) {
        assert_eq!(
            self.pages.lock().insert(page_id, ()),
            None,
            "Did not expect page {page_id:?} to have already been inserted"
        );
    }

    fn read(&self, id: PageId) {
        // TODO(mgoulet): Add assert_matches! when that is introduced
        self.pages.lock().get_refresh(&id);
    }

    fn write(&self, id: PageId) {
        self.pages.lock().get_refresh(&id);
    }
}
