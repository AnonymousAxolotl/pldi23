use std::cell::UnsafeCell;
use std::mem::{ManuallyDrop, MaybeUninit};
use std::ptr;
use std::sync::Barrier;
use std::sync::atomic::{Ordering, AtomicPtr};

pub struct Packet<T> {
    swap: AtomicPtr<ManuallyDrop<T>>,
    barrier: Barrier,
}

impl<T> Packet<T> {

    pub fn new() -> Self {
        Self {
            swap: AtomicPtr::new(ptr::null_mut::<ManuallyDrop<T>>()),
            barrier: Barrier::new(2),
        }
    }

    pub fn send(&self, v: T) {
        let cell = UnsafeCell::new(ManuallyDrop::new(v));
        self.barrier.wait(); // Wait for recv to arrive
        {
            let ptr = cell.get();
            self.swap.compare_exchange(
                ptr::null_mut(), 
                ptr,
                Ordering::SeqCst,
                Ordering::SeqCst
            ).unwrap();
        }
        self.barrier.wait(); // Tell recv we're ready to swap
        self.barrier.wait(); // Recv has taken the value
        // ManuallyDrop means Drop _will not_ be called on v
    }

    pub fn recv(&self) -> T {
        let mut cell = UnsafeCell::new(MaybeUninit::uninit());
        self.barrier.wait(); // Wait for send to arrive
        self.barrier.wait(); // Wait for send to swap pointer
        unsafe {
            let src = self.swap.load(Ordering::SeqCst);
            debug_assert!(src != ptr::null_mut());
            let dst = cell.get();
            ptr::copy_nonoverlapping::<T>(src as *const T, dst as *mut T, 1);
            let val = cell.get_mut();
            self.barrier.wait(); // Tell send we got the value
            val.assume_init_read()
        }
    }

    pub fn reset(&self) {
        self.swap.store(ptr::null_mut::<ManuallyDrop<T>>(), Ordering::SeqCst);
    }

    // pub fn transmute<U>(self) -> Packet<U> {
    //     self.reset();
    //     unsafe { transmute(self) }
    // }


}