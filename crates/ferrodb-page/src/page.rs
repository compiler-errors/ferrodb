use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;
use std::sync::atomic::{AtomicUsize, Ordering};

use ferrodb_util::id_type;
use parking_lot::{
    MappedRwLockReadGuard, MappedRwLockWriteGuard, RwLock, RwLockReadGuard, RwLockWriteGuard,
};
use thiserror::Error;

use crate::replacement_strategy::ReplacementStrategy;

id_type!(pub PageId);

static ID: AtomicUsize = AtomicUsize::new(0);

pub(crate) struct Page {
    pub id: PageId,
    inner: NonNull<PageInner>,
}

#[derive(Debug, Error)]
pub(crate) enum PageCannotBeInvalidated {
    #[error("Page is still pinned, so it cannot be invalidated")]
    StillPinned,
    #[error("Page is already invalidated, cannot be invalidated twice")]
    AlreadyInvalidated,
}

impl Page {
    pub fn allocate_with_size(
        size: usize,
        strat: &'static dyn ReplacementStrategy,
    ) -> (Page, PageHandle, PageRef) {
        Self::allocate(vec![0; size].into(), strat)
    }

    pub fn allocate(
        contents: Box<[u8]>,
        strat: &'static dyn ReplacementStrategy,
    ) -> (Page, PageHandle, PageRef) {
        let inner: NonNull<PageInner> = Box::leak(Box::new(PageInner {
            handle_count: AtomicUsize::new(3),
            ref_count: AtomicUsize::new(1),
            payload: RwLock::new(Some(contents)),
        }))
        .into();

        let id = PageId(ID.fetch_add(1, Ordering::SeqCst));

        (
            Page { id, inner },
            PageHandle { id, inner, strat },
            PageRef { id, inner, strat },
        )
    }

    pub fn try_invalidate(&self) -> Result<Box<[u8]>, PageCannotBeInvalidated> {
        match self
            .inner()
            .ref_count
            .compare_exchange(1, 0, Ordering::SeqCst, Ordering::SeqCst)
        {
            Err(0) => return Err(PageCannotBeInvalidated::AlreadyInvalidated),
            Err(_) => return Err(PageCannotBeInvalidated::StillPinned),
            Ok(1) => {},
            Ok(_) => unreachable!(),
        }

        Ok(self
            .inner()
            .payload
            .try_write()
            .expect("Unexpected reader when PageInner.ref_count is zero")
            .take()
            .expect("Page seems to have been invalidated, but we expected it to be valid"))
    }

    fn inner(&self) -> &PageInner {
        // SAFETY: Inner will be valid for at least as long as this struct is alive
        unsafe { &*self.inner.as_ptr() }
    }
}

impl Drop for Page {
    fn drop(&mut self) {
        let count = self.inner().handle_count.fetch_sub(1, Ordering::SeqCst);

        if count == 1 {
            if self.inner().ref_count.load(Ordering::SeqCst) > 0 {
                panic!(
                    "Either we let a stray PageRef escape, or PageHandle has been dropped without \
                     first invalidating the page. This is not good."
                );
            }

            // SAFETY: This pointer was allocated from Box::leak. We're the last ones to
            // have a reference to this pointer.
            let _ = unsafe { Box::from_raw(self.inner.as_ptr()) };
        }
    }
}

pub struct PageHandle {
    id: PageId,
    inner: NonNull<PageInner>,
    strat: &'static dyn ReplacementStrategy,
}

#[derive(Debug, Error)]
#[error("Page is invalidated, so it can't be pinned, and it must be reloaded")]
pub struct PageInvalidated;

impl PageHandle {
    pub fn pin(&self) -> Result<PageRef, PageInvalidated> {
        loop {
            let ref_count = self.inner().ref_count.load(Ordering::SeqCst);

            if ref_count == 0 {
                return Err(PageInvalidated);
            }

            if self
                .inner()
                .ref_count
                .compare_exchange(ref_count, ref_count + 1, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok()
            {
                return Ok(PageRef {
                    id: self.id,
                    inner: self.inner,
                    strat: self.strat,
                });
            }
        }
    }

    fn inner(&self) -> &PageInner {
        // SAFETY: Inner will be valid for at least as long as this struct is alive
        unsafe { &*self.inner.as_ptr() }
    }
}

impl Drop for PageHandle {
    fn drop(&mut self) {
        let count = self.inner().handle_count.fetch_sub(1, Ordering::SeqCst);

        if count == 1 {
            if self.inner().ref_count.load(Ordering::SeqCst) > 0 {
                panic!(
                    "Either we let a stray PageRef escape, or both Page and PageHandle have been \
                     dropped without first invalidating the page. This is not good."
                );
            }

            // SAFETY: This pointer was allocated from Box::leak. We're the last ones to
            // have a reference to this pointer.
            let _ = unsafe { Box::from_raw(self.inner.as_ptr()) };
        }
    }
}

pub struct PageRef {
    id: PageId,
    inner: NonNull<PageInner>,
    strat: &'static dyn ReplacementStrategy,
}

impl PageRef {
    fn inner(&self) -> &PageInner {
        // SAFETY: Inner will be valid for at least as long as this struct is alive
        unsafe { &*self.inner.as_ptr() }
    }

    pub fn read(&self) -> PageReadGuard<'_> {
        let lock = self.inner().payload.read();
        self.strat.read(self.id);

        PageReadGuard(RwLockReadGuard::map(lock, |page| {
            page.as_deref().expect(
                "Expected page to not be invalidated yet, since we still have an open PageRef",
            )
        }))
    }

    pub fn write(&self) -> PageWriteGuard<'_> {
        let lock = self.inner().payload.write();
        self.strat.write(self.id);

        PageWriteGuard(RwLockWriteGuard::map(lock, |page| {
            page.as_deref_mut().expect(
                "Expected page to not be invalidated yet, since we still have an open PageRef",
            )
        }))
    }
}

pub struct PageReadGuard<'a>(MappedRwLockReadGuard<'a, [u8]>);

impl Deref for PageReadGuard<'_> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

pub struct PageWriteGuard<'a>(MappedRwLockWriteGuard<'a, [u8]>);

impl Deref for PageWriteGuard<'_> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl DerefMut for PageWriteGuard<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.deref_mut()
    }
}

impl Clone for PageRef {
    fn clone(&self) -> PageRef {
        self.inner().ref_count.fetch_add(1, Ordering::SeqCst);

        PageRef {
            id: self.id,
            inner: self.inner,
            strat: self.strat,
        }
    }
}

impl Drop for PageRef {
    fn drop(&mut self) {
        self.inner().ref_count.fetch_sub(1, Ordering::SeqCst);
    }
}

struct PageInner {
    handle_count: AtomicUsize,
    ref_count: AtomicUsize,
    payload: RwLock<Option<Box<[u8]>>>,
}
