use rust_pst::{new_chan, Task, task, chans, spawn};
use rust_pst::nat::{ONE, THREE, EIGHTEEN};
use rust_pst::session::{Send, Recv, Prd, P};
use rust_pst::chan::TaskRet::Continue;

use std::thread;
use std::time::Duration;

fn microphone() -> Task![ONE; i32; Prd<ONE, P, Send<u8, P>>] 
{
    task![v; c : Send<u8, P> => { 
        thread::sleep(Duration::from_millis(1));
        let ((), c) = c.send(1);
        Continue(chans![c], v+1)
    }]
}

fn noisegate() -> Task![ONE; (); Prd<ONE, P, Recv<u8, P>>, 
                                 Prd<ONE, P, Send<u8, P>>] 
{
    task![_; c1,c2 : Recv<u8, P>, Send<u8, P> => { 
        let (_, c1) = c1.recv();
        let ((), c2) = c2.send(1);
        Continue(chans![c1, c2], ())
    }]
}

fn lpf() -> Task![THREE; (); Prd<THREE, P, Recv<u8, Recv<u8, Recv<u8, P>>>>, 
                             Prd<THREE, P, Send<i32, P>>] 
{
    task![_; c1, c2 : Recv<u8, Recv<u8, Recv<u8, P>>>, Send<i32, P> => { 
        let (_, c1) = c1.recv();
        let (_, c1) = c1.recv();
        let (_, c1) = c1.recv();
        let ((), c2) = c2.send(1);
        Continue(chans![c1, c2], ())
    }]
}

fn fft() -> Task![EIGHTEEN; ();   Prd<EIGHTEEN, P, Recv<i32, Recv<i32, Recv<i32, Recv<i32, Recv<i32, Recv<i32, P>>>>>>>,
                                  Prd<EIGHTEEN, P, Send<i32, P>>] 
{
    task![_; c1,c2 : Recv<i32, Recv<i32, Recv<i32, Recv<i32, Recv<i32, Recv<i32, P>>>>>>, 
                     Send<i32, P> => { 
        let (_, c1) = c1.recv();
        let (_, c1) = c1.recv();
        let (_, c1) = c1.recv();
        let (_, c1) = c1.recv();
        let (_, c1) = c1.recv();
        let (_, c1) = c1.recv();
        let ((), c2) = c2.send(1);
        Continue(chans![c1, c2], ())
    }]
}

fn nn() -> Task![EIGHTEEN; (); Prd<EIGHTEEN, P, Recv<i32,  P>>]
{
    task![_; c : Recv<i32, P> => {
        let (v1, c) = c.recv();
        println!("{:?}", (v1));
        Continue(chans!(c), ())
    }]
}

type MicOut   = Prd<ONE, P, Send<u8, P>>;
type NgIn     = Prd<ONE, P, Recv<u8, P>>;
type NgOut    = Prd<ONE, P, Send<u8, P>>;
type LpfIn    = Prd<THREE, P, Recv<u8, Recv<u8, Recv<u8, P>>>>;
type LpfOut   = Prd<THREE, P, Send<i32, P>>;
type FftIn    = Prd<EIGHTEEN, P, Recv<i32, Recv<i32, Recv<i32, Recv<i32, Recv<i32, Recv<i32, P>>>>>>>;
type FftOut   = Prd<EIGHTEEN, P, Send<i32, P>>;
type NnIn     = Prd<EIGHTEEN, P, Recv<i32, P>>;

fn main() {
    let (ka1, ka2)= new_chan!(MicOut, NgIn);
    let (kb1, kb2)= new_chan!(NgOut, LpfIn);
    let (kc1, kc2)= new_chan!(LpfOut, FftIn);
    let (kd1, kd2)= new_chan!(FftOut, NnIn);

    let   p1 = spawn(microphone(), || chans!(ka1),            || 0);
    let  _p2 = spawn(noisegate(),  || chans!(ka2, kb1),       || ());
    let  _p3 = spawn(lpf(),        || chans!(kb2, kc1),       || ());
    let  _p4 = spawn(fft(),        || chans!(kc2, kd1),       || ());
    let  _p5 = spawn(nn(),         || chans!(kd2),            || ());

    let _ = p1.join();
}