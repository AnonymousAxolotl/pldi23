use rust_pst::{new_chan, Task, task, chans, spawn};
use rust_pst::nat::{ONE, FIVE, TWENTY};
use rust_pst::session::{Send, Recv, Prd, P};
use rust_pst::chan::TaskRet::Continue;

use std::thread;
use std::time::Duration;

fn accel() -> Task![ONE; i32; Prd<ONE, P, Send<i32, P>>] 
{
    task![v; c : Send<i32, P> => { 
        thread::sleep(Duration::from_millis(5));
        let ((), c) = c.send(v);
        Continue(chans![c], v+1)
    }]
}

fn lpf() -> Task![FIVE; (); Prd<FIVE, P, Recv<i32, Recv<i32, Recv<i32, Recv<i32, Recv<i32, P>>>>>>, 
                            Prd<FIVE, P, Send<i32, P>>] 
{
    task![_; c1,c2 : Recv<i32, Recv<i32, Recv<i32, Recv<i32,  Recv<i32, P>>>>>, Send<i32, P> => { 
        let (_, c1) = c1.recv();
        let (_, c1) = c1.recv();
        let (_, c1) = c1.recv();
        let (_, c1) = c1.recv();
        let (_, c1) = c1.recv();
        let ((), c2) = c2.send(1);
        Continue(chans![c1, c2], ())
    }]
}

fn gyro() -> Task![FIVE; (); Prd<FIVE, P, Send<i32, P>>] 
{
    task![_; c : Send<i32, P> => { 
        let ((), c) = c.send(1);
        Continue(chans![c], ())
    }]
}

fn kf() -> Task![FIVE; ();   Prd<FIVE, P, Recv<i32, P>>,
                             Prd<FIVE, P, Recv<i32, P>>,
                             Prd<FIVE, P, Send<i32, P>>] 
{
    task![_; c1,c2,c3 : Recv<i32, P>, Recv<i32, P>, Send<i32, P> => { 
        let (_, c1) = c1.recv();
        let (_, c2) = c2.recv();
        let ((), c3) = c3.send(1);
        Continue(chans![c1, c2, c3], ())
    }]
}

fn app() -> Task![TWENTY; (); Prd<TWENTY, P, Recv<i32, Recv<i32, Recv<i32, Recv<i32, P>>>>>]
{
    task![_; c : Recv<i32, Recv<i32, Recv<i32, Recv<i32, P>>>> => {
        let (v1, c) = c.recv();
        let (v2, c) = c.recv();
        let (v3, c) = c.recv();
        let (v4, c) = c.recv();
        println!("{:?}", (v1+v2+v3+v4));
        Continue(chans!(c), ())
    }]
}

type AccOut   = Prd<ONE, P, Send<i32, P>>;
type LpfIn    = Prd<FIVE, P, Recv<i32, Recv<i32, Recv<i32, Recv<i32,  Recv<i32, P>>>>>>;
type LpfOut   = Prd<FIVE, P, Send<i32, P>>;
type KfInLPF  = Prd<FIVE, P, Recv<i32, P>>;
type GyroOut  = Prd<FIVE, P, Send<i32, P>>;
type KfInGyro = Prd<FIVE, P, Recv<i32, P>>;
type KfOut    = Prd<FIVE, P, Send<i32, P>>;
type AppIn    = Prd<TWENTY, P, Recv<i32, Recv<i32, Recv<i32,  Recv<i32, P>>>>>;

fn main() {
    let (ka1, ka2)= new_chan!(AccOut, LpfIn);
    let (kb1, kb2)= new_chan!(LpfOut, KfInLPF);
    let (kc1, kc2)= new_chan!(GyroOut, KfInGyro);
    let (kd1, kd2)= new_chan!(KfOut, AppIn);

    let   p1 = spawn(accel(), || chans!(ka1),            || 0);
    let  _p2 = spawn(lpf(),   || chans!(ka2, kb1),       || ());
    let  _p3 = spawn(kf(),    || chans!(kb2, kc2, kd1),  || ());
    let  _p4 = spawn(gyro(),  || chans!(kc1),            || ());
    let  _p5 = spawn(app(),   || chans!(kd2),            || ());

    let _ = p1.join();
}