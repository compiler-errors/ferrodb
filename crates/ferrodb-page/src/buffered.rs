use std::collections::HashMap;

use parking_lot::Mutex;

use crate::page::Page;
use crate::replacement_strategy::{NoPages, ReplacementStrategy};
use crate::{PageHandle, PageId, PageManager, PageRef};

struct BufferedPageManager<R> {
    pages: Mutex<HashMap<PageId, Page>>,
    strat: R,
    limit: usize,
}

impl<R> BufferedPageManager<R>
where
    R: ReplacementStrategy,
{
    fn new(limit: usize) -> Self {
        BufferedPageManager {
            pages: Mutex::default(),
            strat: R::new(limit),
            limit,
        }
    }
}

impl<R> PageManager for BufferedPageManager<R>
where
    R: ReplacementStrategy,
{
    fn allocate(&'static self) -> Result<(PageHandle, PageRef), NoPages> {
        let mut pages = self.pages.lock();

        let (page, page_handle, page_ref) = if pages.len() < self.limit {
            Page::allocate_with_size(crate::page_size(), &self.strat)
        } else {
            let mut invalidated = None;

            let page_id = self.strat.evict(|page_id| {
                if let Ok(buf) = pages[&page_id].try_invalidate() {
                    invalidated = Some(buf);
                    true
                } else {
                    false
                }
            })?;

            let contents =
                invalidated.expect("We evicted a page, so we must have reclaimed a page buffer");

            pages.remove(&page_id);
            // Reallocate the page contents
            Page::allocate(contents, &self.strat)
        };

        pages.insert(page.id, page);
        Ok((page_handle, page_ref))
    }
}
