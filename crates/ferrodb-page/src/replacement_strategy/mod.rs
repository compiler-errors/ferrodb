mod fifo;
mod lru;
mod noop;
mod random;

use thiserror::Error;

pub use self::fifo::FifoReplacementStrategy;
pub use self::lru::LruReplacementStrategy;
pub use self::noop::NoOpReplacementStrategy;
pub use self::random::RandomReplacementStrategy;
pub use crate::PageId;

pub trait ReplacementStrategy {
    fn new(limit: usize) -> Self
    where
        Self: Sized;

    fn evict<F>(&self, try_evict: F) -> Result<PageId, NoPages>
    where
        Self: Sized,
        F: FnMut(PageId) -> bool;

    fn allocate(&self, id: PageId);

    fn read(&self, id: PageId);

    fn write(&self, id: PageId);
}

#[derive(Debug, Error)]
#[error("There are no pages that can be evicted")]
pub struct NoPages;
