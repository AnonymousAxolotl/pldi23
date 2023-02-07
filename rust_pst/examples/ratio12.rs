use rust_pst::{new_chan, Task, task, chans, spawn};
use rust_pst::nat::{ONE, TWO, FOUR, NonZero, Nat};
use rust_pst::session::{Send, Recv, Prd, P};
use rust_pst::chan::TaskRet::Continue;

use std::thread;
use std::time::Duration;
use std::fmt::Debug;
use std::marker::Send as RustSend;

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

fn pair<N, V>() -> Task![N; (); Prd<N, P, Recv<V, Recv<V, P>>>, Prd<N, P, Send<(V, V), P>>]
where
    N: Nat + NonZero,
    V: RustSend
{
    task![_; c1, c2: Recv<V, Recv<V, P>>, Send<(V, V), P> => {
        let (v1, c1) = c1.recv();
        let (v2, c1) = c1.recv();
        let ((), c2) = c2.send((v1, v2));
        Continue(chans!(c1, c2), ())
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
        Continue(chans!(c), ())
    }]
}
    

// Period can be generic

// So 1 : 2 works because rates match
// type ProdPrd = ONE;
// type ConsPrd = TWO;

// But 2 : 4 also works, because rates still match
type ProdPrd = TWO;
type ConsPrd = FOUR;

// See examples/rate12.rs
type ProdOut = Prd<ProdPrd, P, Send<i32, P>>;
type PairIn  = Prd<ConsPrd, P, Recv<i32, Recv<i32, P>>>;
type PairOut = Prd<ConsPrd, P, Send<(i32, i32), P>>;
type ConsIn  = Prd<ConsPrd, P, Recv<(i32, i32), P>>;

fn main() {

    let (ka1, ka2)= new_chan!(ProdOut, PairIn);
    let (kb1, kb2)= new_chan!(PairOut, ConsIn);

    let  p1 = spawn(produce(), || chans!(ka1),      || 42);
    let _p2 = spawn(pair(),    || chans!(ka2, kb1), || ());
    let _p3 = spawn(consume(), || chans!(kb2),      || ());

    let _ = p1.join();

}