use rust_pst::chan::TaskRet::*;
use rust_pst::{new_chan, Task, task, chans, spawn};
use rust_pst::nat::{ONE, NonZero, Nat};
use rust_pst::session::{Send, Recv, Prd, P};

use std::thread;
use std::time::Duration;
use std::fmt::Debug;
use std::marker::Send as RustSend;

// \omega_n t. !V.t
fn produce<N, V>() -> Task![N; V; Prd<N, P, Send<V, P>>] 
where
    N: Nat + NonZero,
    V: RustSend + Copy
{
    task![v; c : Send<V, P> => { 
        thread::sleep(Duration::from_millis(250));
        let ((), c) = c.send(v);
        Continue(chans![c], v)
    }]
}

fn consume<N, V>() -> Task![N; (); Prd<N, P, Recv<V, P>>]
where
    N: Nat + NonZero,
    V: RustSend + Debug
{
    task![_; c : Recv<V, P> => {
        let (v, c) = c.recv();
        println!("{:?}", v);
        Continue(chans![c], ())
    }]
}

type ProdPrd = ONE;
type ConsPrd = ONE;

// See examples/rate12.rs
type ProdOut = Prd<ProdPrd, P, Send<i32, P>>;
type ConsIn  = Prd<ConsPrd, P, Recv<i32, P>>;

fn main() {

    let (ka1, ka2) = new_chan!(ProdOut, ConsIn);

    let  p1 = spawn(produce(), || chans!(ka1),      || 42);
    let _p3 = spawn(consume(), || chans!(ka2),      || ());

    let _ = p1.join();

}