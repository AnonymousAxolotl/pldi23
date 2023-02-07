use rust_pst::{new_chan, Task, task, chans, spawn};
// use rust_pst::nat::{ONE, TWO, Nat, NonZero};
use rust_pst::nat::{ONE, TWO};
use rust_pst::session::{Send, Recv, Prd, P};
use rust_pst::chan::TaskRet::Continue;

use std::thread;
use std::time::Duration;

fn produce() -> Task![ONE; i32; Prd<ONE, P, Send<i32, P>>] 
{
    task![v; c : Send<i32, P> => { 
        thread::sleep(Duration::from_millis(250));
        let ((), c) = c.send(v);
        Continue(chans![c], v+1)
    }]
}

fn consume() -> Task![TWO; (); Prd<TWO, P, Recv<i32, Recv<i32, P>>>]
{
    task![_; c : Recv<i32, Recv<i32, P>> => {
        let (v1, c) = c.recv();
        let (v2, c) = c.recv();
        println!("{:?}", (v1, v2));
        Continue(chans!(c), ())
    }]
}

// Prod_{Out} = \periodic[t]{1}{\nodeadline+1}{!\texttt{int}.t} = \omega_{1} t. !int. t
type ProdOut = Prd<ONE, P, Send<i32, P>>;

// Cons_{In} = \periodic[t]{2}{\nodeadline+2}{?(\texttt{int}, \texttt{int}).t} = \omega_{2} t. ?(int, int). t
type ConsIn  = Prd<TWO, P, Recv<i32, Recv<i32, P>>>;

fn main() {
    let (ka1, ka2)= new_chan!(ProdOut, ConsIn);

    let   p1 = spawn(produce(), || chans!(ka1),      || 0);
    let  _p2 = spawn(consume(), || chans!(ka2),      || ());

    let _ = p1.join();
}