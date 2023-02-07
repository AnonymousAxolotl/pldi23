#![allow(dead_code)]

use core::cell::UnsafeCell;
use core::mem::{ManuallyDrop, MaybeUninit, self};
use core::ptr;
use core::sync::atomic::{Ordering, AtomicPtr};

use freertos_rust::{Semaphore, Duration};

pub struct Packet<T> {
    swap: AtomicPtr<ManuallyDrop<T>>,
    send_sem: Semaphore,
    recv_sem: Semaphore,
}

impl<T> Packet<T> {

    pub fn new() -> Self {
        Self {
            swap: AtomicPtr::new(ptr::null_mut::<ManuallyDrop<T>>()),
            send_sem: Semaphore::new_binary().unwrap(),
            recv_sem: Semaphore::new_binary().unwrap(),
        }
    }

    pub fn send(&self, v: T) {
        let cell = UnsafeCell::new(ManuallyDrop::new(v));
        let _ = self.send_sem.take(Duration::infinite()); // Wait for recv to arrive
        {
            let ptr = cell.get();
            self.swap.compare_exchange(
                ptr::null_mut(), 
                ptr,
                Ordering::SeqCst,
                Ordering::SeqCst
            ).unwrap();
        }
        self.recv_sem.give(); // Tell recv we're ready to swap
        let _ = self.send_sem.take(Duration::infinite()); // Recv has taken the value
        self.recv_sem.give(); // Resync to avoid double give
        // ManuallyDrop means Drop _will not_ be called on v
        
    }

    pub fn recv(&self) -> T {
        let mut cell = UnsafeCell::new(MaybeUninit::uninit());
        self.send_sem.give(); // Tell send we're ready
        let _ = self.recv_sem.take(Duration::infinite()); // Wait for send to swap pointer
        let x = unsafe {
            let src = self.swap.load(Ordering::SeqCst);
            debug_assert!(src != ptr::null_mut());
            let dst = cell.get();
            ptr::copy_nonoverlapping::<T>(src as *const T, dst as *mut T, 1);
            let val = cell.get_mut();
            self.send_sem.give(); // Tell send we got the value
            val.assume_init_read()
        };
        let _ = self.recv_sem.take(Duration::infinite()); // Resync to avoid double give
        x
    }

    pub fn reset(&self) {
        self.swap.store(ptr::null_mut::<ManuallyDrop<T>>(), Ordering::SeqCst);
    }

    pub fn transmute<U>(self) -> Packet<U> {
        self.reset();
        unsafe { mem::transmute(self) }
    }

    pub unsafe fn transmute_ref<U>(&self) -> &Packet<U> {
        self.reset();
        mem::transmute(self)
    }


}