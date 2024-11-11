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
            let value = self.ptr_w.swap(ptr::null_mut(), Ordering::Release);

            f(unsafe { &mut *value });

            self.ptr_r.store(value.clone(), Ordering::Release);

            let old_ptr_w = self.ptr_w.swap(value, Ordering::Release);

            if !old_ptr_w.is_null() {
                unsafe {
                    drop(Box::from_raw(old_ptr_w));
                }
            }
            return true;
        }
        false
    }
}
