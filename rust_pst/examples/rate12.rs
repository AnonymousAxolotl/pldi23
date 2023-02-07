use rust_pst::{new_chan, Task, task, chans, spawn};
use rust_pst::nat::{ONE, TWO, Nat, NonZero};
use rust_pst::session::{Send, Recv, Prd, P};
use rust_pst::chan::TaskRet::Continue;

use std::thread;
use std::time::Duration;
use std::fmt::Debug;
use std::marker::Send as RustSend;

fn produce<V>() -> Task![ONE; V; Prd<ONE, P, Send<V, P>>] 
where
    V: RustSend + Copy
{
    task![v; c : Send<V, P> => { 
        thread::sleep(Duration::from_millis(250));
        let ((), c) = c.send(v);
        Continue(chans![c], v)
    }]
}

fn pair<V>() -> Task![TWO; (); Prd<TWO, P, Recv<V, Recv<V, P>>>, Prd<TWO, P, Send<(V, V), P>>]
where
    V: RustSend
{
    task![_; c1, c2: Recv<V, Recv<V, P>>, Send<(V, V), P> => {
        let (v1, c1) = c1.recv();
        let (v2, c1) = c1.recv();
        let ((), c2) = c2.send((v1, v2));
        Continue(chans!(c1, c2), ())
    }]
}

fn consume<V, N>() -> Task![N; (); Prd<N, P, Recv<V, P>>]
where
    V: RustSend + Debug,
    N: Nat + NonZero
{
    task![_; c : Recv<V, P> => {
        let (v, c) = c.recv();
        println!("{:?}", v);
        Continue(chans!(c), ())
    }]
}

// Prod_{Out} = \periodic[t]{1}{\nodeadline+1}{!\texttt{int}.t} = \omega_{1} t. !int. t
type ProdOut = Prd<ONE, P, Send<i32, P>>;

// Pair_{In} = \periodic[t]{2}{\nodeadline+2}{?\texttt{int}. ?\texttt{int}.t} = \omega_{2} t. ?int. ?int. t
type PairIn  = Prd<TWO, P, Recv<i32, Recv<i32, P>>>;

// Pair_{Out} = \periodic[t]{2}{\nodeadline+2}{!(\texttt{int}, \texttt{int}).t} = \omega_{2} t. !(int, int). t
type PairOut = Prd<TWO, P, Send<(i32, i32), P>>;

// Cons_{In} = \periodic[t]{2}{\nodeadline+2}{?(\texttt{int}, \texttt{int}).t} = \omega_{2} t. ?(int, int). t
type ConsIn  = Prd<TWO, P, Recv<(i32, i32), P>>;

fn main() {

    let (ka1, ka2)= new_chan!(ProdOut, PairIn);
    let (kb1, kb2)= new_chan!(PairOut, ConsIn);

    let  p1 = spawn(produce(), || chans!(ka1),      || 42);
    let _p2 = spawn(pair(),    || chans!(ka2, kb1), || ());
    let _p3 = spawn(consume(), || chans!(kb2),      || ());

    let _ = p1.join();

}