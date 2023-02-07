use rust_pst::{new_chan, Task, task, chans, spawn};
use rust_pst::nat::{ONE, FOUR};
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

fn f(_:(i32, i32, i32, i32)) -> f32 {1.0}

fn lpf() -> Task![FOUR; (); Prd<FOUR, P, Recv<i32, Recv<i32, Recv<i32, Recv<i32, P>>>>>, 
                            Prd<FOUR, P, Send<f32, P>>] 
{
    task![_; c1,c2 : Recv<i32, Recv<i32, Recv<i32, Recv<i32,  P>>>>, Send<f32, P> => { 
        let (v1, c1) = c1.recv();
        let (v2, c1) = c1.recv();
        let (v3, c1) = c1.recv();
        let (v4, c1) = c1.recv();
        let ((), c2) = c2.send(f((v1, v2, v3, v4)));
        Continue(chans![c1, c2], ())
    }]
}

fn app() -> Task![FOUR; (); Prd<FOUR, P, Recv<f32, P>>]
{
    task![_; c : Recv<f32, P> => {
        let (v1, c) = c.recv();
        println!("{:?}", (v1));
        Continue(chans!(c), ())
    }]
}

type AccOut = Prd<ONE, P, Send<i32, P>>;
type LpfIn  = Prd<FOUR, P, Recv<i32, Recv<i32, Recv<i32, Recv<i32,  P>>>>>;
type LpfOut = Prd<FOUR, P, Send<f32, P>>;
type AppIn  = Prd<FOUR, P, Recv<f32, P>>;

fn main() {
    let (ka1, ka2)= new_chan!(AccOut, LpfIn);
    let (kb1, kb2)= new_chan!(LpfOut, AppIn);

    let   p1 = spawn(accel(), || chans!(ka1),      || 0);
    let  _p2 = spawn(lpf(),   || chans!(ka2, kb1), || ());
    let  _p3 = spawn(app(),   || chans!(kb2),      || ());

    let _ = p1.join();
}