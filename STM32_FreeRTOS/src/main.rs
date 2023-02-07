#![no_std]
#![no_main]
// For allocator
#![feature(lang_items)]
#![feature(alloc_error_handler)]

#![allow(dead_code)]

mod macros;
mod nat;
mod session;
mod chan;
mod pkt;
mod ppg;
mod ppg_tasks;
mod util;
mod icons;

extern crate alloc;

use alloc::boxed::Box;
use cortex_m::asm;
use cortex_m_rt::exception;
use cortex_m_rt::{entry, ExceptionFrame};
use embedded_graphics::Drawable;
use embedded_graphics::image::{Image, ImageRaw};
use embedded_graphics::mono_font::MonoTextStyleBuilder;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::Point;
use embedded_graphics::text::{Text, Baseline};
use embedded_hal::digital::v2::OutputPin;
use freertos_rust::*;
use hal::gpio::gpiob::{PB6, PB7};
use hal::i2c::I2c;
use hal::pac::I2C1;
use max3010x::marker::ic::Max30102;
use max3010x::marker::mode::HeartRate;
use pkt::Packet;
use profont::{PROFONT_24_POINT, PROFONT_12_POINT};
use shared_bus::I2cProxy;
use ssd1306::mode::BufferedGraphicsMode;
use ssd1306::{size::DisplaySize128x64, rotation::DisplayRotation};
use ssd1306::{Ssd1306, I2CDisplayInterface, prelude::*};
use util::FreeRTOSBusMutex;
use core::alloc::Layout;
use core::mem::MaybeUninit;
use core::str;
use core::ptr::null_mut;
use core::marker::Send as RustSend;
use core::sync::atomic::{Ordering, AtomicPtr, fence};
use stm32f4xx_hal::gpio::*;
use rtt_target::{rprintln, rtt_init_default};

use cortex_m;
use stm32f4xx_hal as hal;
use stm32f4xx_hal::prelude::*;

use crate::chan::register;
use crate::ppg::{Biquad, AGC, PPG};
use crate::ppg_tasks::{task_decim, task_iir, task_agc, task_ppg};

use max3010x::{Led as MaxLed, Max3010x, SamplingRate};

extern crate panic_halt; // panic handler

use chan::TaskRet::Continue;
use nat::{ONE, TWO, NonZero, Nat, EIGHT};
use session::{Send, Recv, Prd, P, Protocol};

#[global_allocator]
static GLOBAL: FreeRtosAllocator = FreeRtosAllocator;

type TX1F1 = Prd<ONE, P, Send<Option<f32>, P>>;
type TX1F2 = Prd<TWO, P, Send<Option<f32>, P>>;
type TXDF2 = Prd<TWO, P, Send<(bool, f32, bool), P>>;

type RX2F2 = Prd<TWO, P, Recv<Option<f32>, Recv<Option<f32>, P>>>;
type RX1F2 = <TX1F2 as Protocol>::DUAL;
type RXDF8 = Prd<EIGHT, P, Recv<(bool, f32, bool), Recv<(bool, f32, bool), Recv<(bool, f32, bool), Recv<(bool, f32, bool), P>>>>>;


type I2C = I2c<I2C1, (PB6<AlternateOD<AF4>>, PB7<AlternateOD<AF4>>)>;
type I2CBus = I2cProxy<'static, FreeRTOSBusMutex<I2C>>;
type Device = Max3010x<I2CBus, Max30102, HeartRate>;

static RTT_MUTEX: AtomicPtr<Mutex<()>> = AtomicPtr::new(null_mut());

static mut PKTA: MaybeUninit<Packet<TX1F1>> = MaybeUninit::uninit();
static mut PKTB: MaybeUninit<Packet<TX1F2>> = MaybeUninit::uninit();
static mut PKTC: MaybeUninit<Packet<TX1F2>> = MaybeUninit::uninit();
static mut PKTD: MaybeUninit<Packet<TX1F2>> = MaybeUninit::uninit();
static mut PKTE: MaybeUninit<Packet<TX1F2>> = MaybeUninit::uninit();
static mut PKTF: MaybeUninit<Packet<TXDF2>> = MaybeUninit::uninit();

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

fn init_ppg(i2c: I2CBus) -> Device {
    let mut max30102 = Max3010x::new_max30102(i2c);
    max30102.reset().unwrap();

    let mut max30102 = max30102.into_heart_rate().unwrap();
    max30102.set_pulse_amplitude(MaxLed::All, 15).unwrap();
    max30102.enable_fifo_rollover().unwrap();

    max30102.set_sampling_rate(SamplingRate::Sps50).unwrap();

    max30102
}

fn read_sensor() -> Task![ONE; Device; Prd<ONE, P, Send<Option<f32>, P>>] 
{
    task![mut dev; c : Send<Option<f32>, P> => { 
        let x = loop {
            let mut data = [0; 1];
            // break 1.0;
            let nread = dev.read_fifo(&mut data).unwrap_or(0xFF);
            if nread != 0 {
                break (data[0] as f32);
            }
            freertos_rust::CurrentTask::delay(Duration::ms(2));
        };
        // rprintln!("Read {}", x);
        let ((), c) = c.send(if x > 8000.0 { Some(x) } else { None });
        Continue(chans![c], dev)
    }]
}

struct DispState {
    disp: Ssd1306<I2CInterface<I2CBus>, DisplaySize128x64, BufferedGraphicsMode<DisplaySize128x64>>,
    no_finger: ImageRaw<'static, BinaryColor>,
    heart: ImageRaw<'static, BinaryColor>,
    heartbeat: ImageRaw<'static, BinaryColor>,
}

fn init_disp(i2c: I2CBus) -> DispState {
    let interface = I2CDisplayInterface::new(i2c);
    let mut disp = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate90)
        .into_buffered_graphics_mode();
    disp.init().unwrap();
    // disp.flush().unwrap();s
    
    let text_style = MonoTextStyleBuilder::new()
    .font(&PROFONT_24_POINT)
    .text_color(BinaryColor::On)
    .build();

    disp.clear();
    Text::with_baseline("    ", Point::new(0, (64 - 24)/2), text_style, Baseline::Top)
        .draw(&mut disp)
        .unwrap();

    let raw_cross = ImageRaw::<BinaryColor>::new(icons::CROSS_DATA, 64);
    let _cross = Image::new(&raw_cross, Point::new(0, 64));
    let raw_large_heart = ImageRaw::<BinaryColor>::new(icons::LARGE_HEART_DATA, 64);
    let _large_heart = Image::new(&raw_large_heart, Point::new(0, 64));
    let raw_small_heart = ImageRaw::<BinaryColor>::new(icons::SMALL_HEART_DATA, 64);
    let _small_heart = Image::new(&raw_small_heart, Point::new(0, 64));
    let raw_xsmall_heart = ImageRaw::<BinaryColor>::new(icons::XSMALL_HEART_DATA, 64);
    let _xsmall_heart = Image::new(&raw_xsmall_heart, Point::new(0, 64));


    _cross.draw(&mut disp).unwrap();

    disp.flush().unwrap();

    DispState {
        disp,
        no_finger : raw_cross,
        heart : raw_xsmall_heart,
        heartbeat : raw_small_heart,
    }
}

fn task_disp() -> Task![EIGHT; DispState; Prd<EIGHT, P, Recv<(bool, f32, bool), Recv<(bool, f32, bool), Recv<(bool, f32, bool), Recv<(bool, f32, bool), P>>>>>] 
{
    task![mut state; c : Recv<(bool, f32, bool), Recv<(bool, f32, bool), Recv<(bool, f32, bool), Recv<(bool, f32, bool), P>>>> => { 
        let (_, c) = c.recv();
        let (_, c) = c.recv();
        let (_, c) = c.recv();
        let ((detect, bpm_f32, beat), c) = c.recv();

        let bpm = (bpm_f32 + 0.5) as i32;

        let disp = &mut state.disp;
        disp.clear();

        let text_style = MonoTextStyleBuilder::new()
        .font(&PROFONT_24_POINT)
        .text_color(BinaryColor::On)
        .build();

        // Render measurement
        if bpm <= 0 {
            Text::with_baseline("--", Point::new(16, 16), text_style, Baseline::Top).draw(disp)
        } else {
            let digit100: u8 = (bpm / 100).try_into().unwrap();
            let digit10: u8 = ((bpm - digit100 as i32*100) / 10).try_into().unwrap();
            let digit1: u8 = (bpm - digit100 as i32*100 - digit10 as i32*10).try_into().unwrap();

            if digit100 > 0 {
                let buf: &[u8] = &[digit100 + 0x30, digit10 + 0x30, digit1 + 0x30];
                Text::with_baseline(str::from_utf8(buf).unwrap(), Point::new(8, 16), text_style, Baseline::Top).draw(disp)
            } else {
                let buf: &[u8] = &[digit10 + 0x30, digit1 + 0x30];
                Text::with_baseline(str::from_utf8(buf).unwrap(), Point::new(16, 16), text_style, Baseline::Top).draw(disp)
            }
        }.unwrap();

        let label_style = MonoTextStyleBuilder::new()
        .font(&PROFONT_12_POINT)
        .text_color(BinaryColor::On)
        .build();
        Text::with_baseline("bpm", Point::new(21, 42), label_style, Baseline::Top).draw(disp).unwrap();

        if detect {
            if beat {
                Image::new(&state.heartbeat, Point::new(0, 64)).draw(disp).unwrap();
            } else {
                Image::new(&state.heart, Point::new(0, 64)).draw(disp).unwrap();
            }
        } else {
            Image::new(&state.no_finger, Point::new(0, 64)).draw(disp).unwrap();
        }

        disp.flush().unwrap();

        Continue(chans!(c), state)
    }]
}

#[entry]
fn main() -> ! {
    rtt_init();
    rprintln!("Starting...");
    let stm32_periph = hal::stm32::Peripherals::take().unwrap();

    // let gpio_a = stm32_periph.GPIOA.split();
    
    // Configure clocks
    let rcc = stm32_periph.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(160.mhz()).freeze();

    // let led = LED::from_pins(gpio_a.pa1.into_push_pull_output());

    let gpiob = stm32_periph.GPIOB.split();
    let scl = gpiob.pb6.into_pull_up_input().into_alternate_af4_open_drain();
    let sda = gpiob.pb7.into_pull_up_input().into_alternate_af4_open_drain();

    let i2c = hal::i2c::I2c::new(
        stm32_periph.I2C1,
        (scl, sda),
        400.khz(),
        clocks,
    );

    let bus: &'static _ = new_freertos_bus!(I2C = i2c).unwrap();

    let hpf = Biquad::new([0.87033078, -1.74066156, 0.87033078], [-1.72377617, 0.75754694]);
    let agc = AGC::new(400.0, 0.971, 2.0);
    let lpf = Biquad::new([0.11595249, 0.23190498, 0.11595249], [-0.72168143, 0.18549138]);

    // Safety: Unsafe because mutable globals are unsafe w/ mutliple threads
    // We take exclusive mutable borrows here on the main thread only
    let pkt_a = unsafe { PKTA.write(Packet::new()) };
    let pkt_b = unsafe { PKTB.write(Packet::new()) };
    let pkt_c = unsafe { PKTC.write(Packet::new()) };
    let pkt_d = unsafe { PKTD.write(Packet::new()) };
    let pkt_e = unsafe { PKTE.write(Packet::new()) };
    let pkt_f = unsafe { PKTF.write(Packet::new()) };


    let (ka1, ka2) = new_chan!(TX1F1, RX2F2, pkt_a);
    let (kb1, kb2) = new_chan!(TX1F2, RX1F2, pkt_b);
    let (kc1, kc2) = new_chan!(TX1F2, RX1F2, pkt_c);
    let (kd1, kd2) = new_chan!(TX1F2, RX1F2, pkt_d);
    let (ke1, ke2) = new_chan!(TX1F2, RX1F2, pkt_e);
    let (kf1, kf2) = new_chan!(TXDF2, RXDF8, pkt_f);

    register("sensor", 1024, read_sensor(),     || chans!(ka1),         || init_ppg(bus.acquire_i2c()));
    register("decim", 512,  task_decim(),       || chans!(ka2, kb1),    || ());
    register("hpf", 512,    task_iir::<TWO>(),  || chans!(kb2, kc1),    || hpf);
    register("agc", 512,    task_agc::<TWO>(),  || chans!(kc2, kd1),    || agc);
    register("lpf", 512,    task_iir::<TWO>(),  || chans!(kd2, ke1),    || lpf);
    register("ppg", 4096,   task_ppg::<TWO>(),  || chans!(ke2, kf1),    || (PPG::new(), 0.0));
    register("disp", 4096,  task_disp(),        || chans!(kf2),         || init_disp(bus.acquire_i2c()));

    FreeRtosUtils::start_scheduler();
}










#[allow(non_snake_case)]
#[exception]
fn DefaultHandler(_irqn: i16) {
// custom default handler
// irqn is negative for Cortex-M exceptions
// irqn is positive for device specific (line IRQ)
// set_led(true);(true);
// panic!("Exception: {}", irqn);
}

#[allow(non_snake_case)]
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

#[allow(non_snake_case)]
#[no_mangle]
fn vApplicationStackOverflowHook(_pxTask: FreeRtosTaskHandle, _pcTaskName: FreeRtosCharPtr) {
    asm::bkpt();
}
