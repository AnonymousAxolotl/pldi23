use rust_pst::{new_chan, Task, task, chans, spawn};
use rust_pst::nat::{Nat, NonZero, ONE};
use rust_pst::session::{Send, Recv, Prd, P};
use rust_pst::chan::TaskRet::Continue;

use std::process::exit;
use std::time::Instant;
use std::fmt::Debug;
use std::marker::Send as RustSend;

fn zero_source() -> Task![ONE; u32; Prd<ONE, P, Send<i32, Send<i32, Send<i32, P>>>>] 
{
    task![i; c : Send<i32, Send<i32, Send<i32, P>>> => { 
        if i == 0 {
            exit(0);
        }
        let ((), c) = c.send(0);
        let start = Instant::now();
            let ((), c) = c.send(0);
        let elapsed = start.elapsed();
        println!("{}", elapsed.as_nanos());
        let ((), c) = c.send(0);
        Continue(chans![c], i-1)
    }]
}

fn null_sink2<V, N>() -> Task![N; (); Prd<N, P, Recv<V, Recv<V, Recv<V, P>>>>]
where
    V: RustSend + Debug,
    N: Nat + NonZero
{
    task![_; c : Recv<V, Recv<V, Recv<V, P>>> => {
        let (_, c) = c.recv();
        let (_, c) = c.recv();
        let (_, c) = c.recv();
        Continue(chans!(c), ())
    }]
}

// Prod_{Out} = \periodic[t]{1}{\nodeadline+1}{!\texttt{int}.t} = \omega_{1} t. !int. t
type ProdOut = Prd<ONE, P, Send<i32, Send<i32, Send<i32, P>>>>;
type ConsIn  = Prd<ONE, P, Recv<i32, Recv<i32, Recv<i32, P>>>>;


// cargo run --release --example microbench | tee times.txt | sort -n | head -n 50000 | tail -n 1
fn main() {

    let (k1, k2)= new_chan!(ProdOut, ConsIn);

    let  p1 = spawn(zero_source(), || chans!(k1),      || 100_000);
    let _p2 = spawn(null_sink2(),  || chans!(k2),      || ());

    let _ = p1.join();

}