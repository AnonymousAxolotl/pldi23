use freertos_rust::{Mutex, Duration};

pub struct FreeRTOSBusMutex<T>(Mutex<T>);

unsafe impl<T> Send for FreeRTOSBusMutex<T> { }
unsafe impl<T> Sync for FreeRTOSBusMutex<T> { }

impl<T> shared_bus::BusMutex for FreeRTOSBusMutex<T> {
    type Bus = T;

    fn create(v: T) -> Self {
        Self(Mutex::new(v).unwrap())
    }

    fn lock<R, F: FnOnce(&mut Self::Bus) -> R>(&self, f: F) -> R {
        let mut v = self.0.lock(Duration::infinite()).unwrap();
        let x = f(&mut v);
        let _ = v;
        x
    }
}

pub type FreeRTOSBusManager<BUS> = shared_bus::BusManager<FreeRTOSBusMutex<BUS>>;

#[macro_export]
macro_rules! new_freertos_bus {
    ($bus_type:ty = $bus:expr) => {{
        let m: Option<&'static mut _> = $crate::cortex_m::singleton!(
            : $crate::util::FreeRTOSBusManager<$bus_type> =
                $crate::util::FreeRTOSBusManager::new($bus)
        );
        m
    }};
}