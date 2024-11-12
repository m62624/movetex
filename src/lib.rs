use std::ptr;
use std::sync::atomic::{AtomicPtr, Ordering};

pub struct Movetex<T: Clone> {
    ptr_r: AtomicPtr<T>,
    ptr_w: AtomicPtr<T>,
}

impl<T: Clone> Movetex<T> {
    pub fn new(value: T) -> Self {
        Self {
            ptr_r: AtomicPtr::new(Box::into_raw(Box::new(value.clone()))),
            ptr_w: AtomicPtr::new(Box::into_raw(Box::new(value))),
        }
    }

    pub fn read(&self) -> &T {
        unsafe { &*self.ptr_r.load(Ordering::Acquire) }
    }

    pub fn write(&self, f: impl FnOnce(&mut T)) -> bool {
        if !self.ptr_w.load(Ordering::Acquire).is_null() {
            let mut value =
                unsafe { *Box::from_raw(self.ptr_w.swap(ptr::null_mut(), Ordering::Release)) };

            f(&mut value);

            drop(unsafe {
                Box::from_raw(
                    self.ptr_r
                        .swap(Box::into_raw(Box::new(value.clone())), Ordering::Release),
                )
            });

            self.ptr_w
                .store(Box::into_raw(Box::new(value)), Ordering::Release);

            return true;
        }
        false
    }

    pub fn swap(&self, value: T) -> Option<T> {
        let ptr = self
            .ptr_w
            .swap(Box::into_raw(Box::new(value)), Ordering::Release);
        if ptr.is_null() {
            return None;
        }
        Some(unsafe { *Box::from_raw(ptr) })
    }
}

impl<T: Clone> Drop for Movetex<T> {
    fn drop(&mut self) {
        unsafe {
            let ptr_r = self.ptr_r.load(Ordering::Acquire);
            if !ptr_r.is_null() {
                drop(Box::from_raw(ptr_r));
            }
            let ptr_w = self.ptr_w.load(Ordering::Acquire);
            if !ptr_w.is_null() {
                drop(Box::from_raw(ptr_w));
            }
        }
    }
}
