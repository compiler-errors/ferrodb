use crate::page::Page;
use crate::replacement_strategy::NoOpReplacementStrategy;
use crate::{NoPages, PageHandle, PageManager, PageRef};

struct UnlimitedPageManager;

impl PageManager for UnlimitedPageManager {
    fn allocate(&'static self) -> Result<(PageHandle, PageRef), NoPages> {
        // Allocate a page without regards to memory allocation. Drop the Page returned
        // from the inner allocate function because we don't actually need to keep it
        // around for bookkeeping.
        let (page, page_handle, page_ref) =
            Page::allocate_with_size(crate::page_size(), &NoOpReplacementStrategy);

        // Leak this page... (for now, we don't wanna cause panics if we drop the page)
        std::mem::forget(page);

        Ok((page_handle, page_ref))
    }
}
