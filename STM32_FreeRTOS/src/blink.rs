#![no_std]
#![no_main]
// For allocator
#![feature(lang_items)]
#![feature(alloc_error_handler)]

mod macros;
mod nat;
mod session;
mod chan;
mod pkt;


extern crate alloc;

use alloc::boxed::Box;
use cortex_m::asm;
use cortex_m_rt::exception;
use cortex_m_rt::{entry, ExceptionFrame};
use embedded_hal::digital::v2::OutputPin;
use freertos_rust::*;
use pkt::Packet;
use core::alloc::Layout;
use core::mem::MaybeUninit;
use core::ptr::null_mut;
use core::marker::Send as RustSend;
use core::sync::atomic::{Ordering, AtomicPtr, fence};
use stm32f4xx_hal::gpio::*;
use rtt_target::{rprintln, rtt_init_default};

use cortex_m;
use stm32f4xx_hal as hal;

use crate::chan::register;
use crate::hal::{
    stm32::Peripherals,
};

extern crate panic_halt; // panic handler

use chan::TaskRet::Continue;
use nat::{ONE, NonZero, Nat};
use session::{Send, Recv, Prd, P};

#[global_allocator]
static GLOBAL: FreeRtosAllocator = FreeRtosAllocator;

static RTT_MUTEX: AtomicPtr<Mutex<()>> = AtomicPtr::new(null_mut());

static mut PKTA: MaybeUninit<Packet<Prd<ONE, P, Send<i32, P>>>> = MaybeUninit::uninit();
static mut PKTB: MaybeUninit<Packet<Prd<ONE, P, Send<bool, P>>>> = MaybeUninit::uninit();

pub struct LED<D1: OutputPin> {
    d1: D1,
}

impl<D1: OutputPin> LED<D1>
{
    pub fn from_pins(d1: D1) -> LED<D1> {
        LED {
            d1
        }
    }
    pub fn set_led(&mut self, on:bool){
        if on {
            let _ = self.d1.set_high();
        } else {
            let _ = self.d1.set_low();
        }
    }
}

fn critical<F: FnOnce()-> ()>(f: F) {
    let mutex = unsafe { &*RTT_MUTEX.load(Ordering::SeqCst) };
    let h = mutex.lock(Duration::infinite()).unwrap();
    f();
    // I feel like there should be a barrier here?
    fence(Ordering::SeqCst);
    drop(h); // Unlock
}

fn rtt_init() {
    let channels = rtt_init_default!();
    let mutex = Box::new(Mutex::new(()).unwrap());
    RTT_MUTEX.store(Box::leak(mutex), Ordering::SeqCst);
    unsafe {
        rtt_target::set_print_channel_cs(
            channels.up.0,
            &((|arg, f| critical(|| f(arg))) as rtt_target::CriticalSectionFunc),
        );
    };
}



fn produce<N>() -> Task![N; i32; Prd<N, P, Send<i32, P>>] 
where
    N: Nat + NonZero + 'static
{
    task![v; c : Send<i32, P> => { 
        freertos_rust::CurrentTask::delay(Duration::ms(500));
        let ((), c) = c.send(v);
        rprintln!("Send {:?}", v);
        Continue(chans![c], v+1)
    }]
}

fn consume<N>() -> Task![N; (); Prd<N, P, Recv<i32, P>>]
where
    N: Nat + NonZero + 'static
{
    task![_; c : Recv<i32, P> => {
        let (v, c) = c.recv();
        rprintln!("Recv {:?}", v);
        Continue(chans![c], ())
    }]
}

fn toggle() -> Task![ONE; bool; Prd<ONE, P, Send<bool, P>>] 
{
    task![v; c : Send<bool, P> => { 
        freertos_rust::CurrentTask::delay(Duration::ms(500));
        let ((), c) = c.send(v);
        Continue(chans![c], !v)
    }]
}

fn set_led<N, O>() -> Task![N; LED<O>; Prd<N, P, Recv<bool, P>>]
where
    N: Nat + NonZero + 'static,
    O: OutputPin + RustSend
{
    task![mut led; c : Recv<bool, P> => {
        let (v, c) = c.recv();
        rprintln!("LED {:?}", if v {"ON"} else {"OFF"});
        led.set_led(v);
        Continue(chans![c], led)
    }]
}

#[entry]
fn main() -> ! {
    rtt_init();
    rprintln!("Starting...");
    let dp = Peripherals::take().unwrap();
    let gpioa = dp.GPIOA.split();
    let led = LED::from_pins(gpioa.pa1.into_push_pull_output());

    // Safety: Unsafe because mutable globals are unsafe w/ mutliple threads
    // We take exclusive mutable borrows here on the main thread only
    let mut pkt_a = unsafe { PKTA.write(Packet::new()) };
    let mut pkt_b = unsafe { PKTB.write(Packet::new()) };

    let (ka1, ka2) = new_chan!(Prd<ONE, P, Send<i32, P>>, Prd<ONE, P, Recv<i32, P>>, pkt_a);

    register("produce", 512, produce(), || chans![ka1], || 0);
    register("consume", 512, consume(), || chans![ka2], || ());

    let (kb1, kb2) = new_chan!(Prd<ONE, P, Send<bool, P>>, Prd<ONE, P, Recv<bool, P>>, pkt_b);

    register("toggle", 512, toggle(), || chans![kb1], || true);
    register("set_led", 512, set_led(), || chans![kb2], || led);

    FreeRtosUtils::start_scheduler();
}











#[exception]
fn DefaultHandler(_irqn: i16) {
// custom default handler
// irqn is negative for Cortex-M exceptions
// irqn is positive for device specific (line IRQ)
// set_led(true);(true);
// panic!("Exception: {}", irqn);
}

#[exception]
fn HardFault(_ef: &ExceptionFrame) -> ! {
// Blink 3 times long when exception occures
    for _ in 0..3 {
        // set_led(true);
        // delay_n(1000);
        // set_led(false);
        // delay_n(555);
    }
    loop {}
}

// define what happens in an Out Of Memory (OOM) condition
#[alloc_error_handler]
fn alloc_error(_layout: Layout) -> ! {
    //set_led(true);
    asm::bkpt();
    loop {}
}

#[no_mangle]
fn vApplicationStackOverflowHook(_pxTask: FreeRtosTaskHandle, _pcTaskName: FreeRtosCharPtr) {
    asm::bkpt();
}
