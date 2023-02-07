#![allow(dead_code)]

use rust_pst::chan::{RecvAll, Chan};
use rust_pst::{Task, task, chans};
use rust_pst::nat::{Nat, NonZero};
use rust_pst::session::{Send, Recv, P, Prd, CanExpand, Expansion, HasCanon};
use rust_pst::chan::TaskRet::Continue;
use std::marker::Send as RustSend;


fn vectorize<N, V, R>() -> Task![N; (); Prd<N, P, <<< R as CanExpand<Recv<V, P>> >::EXP as Expansion<R, Recv<V, P>>>::EXPANDED as HasCanon>::CANON >, Prd<N, P, Send<Vec<V>, P>>]
where
    N: Nat + NonZero,
    V: RustSend,
    R: Nat + NonZero + CanExpand<Recv<V, P>>,
    Chan<<<< R as CanExpand<Recv<V, P>> >::EXP as Expansion<R, Recv<V, P>>>::EXPANDED as HasCanon>::CANON > : RecvAll<V, P>, <<R as CanExpand<Recv<V, P>>>::EXP as Expansion<R, Recv<V, P>>>::EXPANDED: HasCanon
{
    task![_; c1, c2 : <<< R as CanExpand<Recv<V, P>> >::EXP as Expansion<R, Recv<V, P>>>::EXPANDED as HasCanon>::CANON , Send<Vec<V>, P> => {
        let (v, p) = c1.recv_all();
        let ((), c2) = c2.send(v);
        Continue(chans![p, c2], ())
    }]
}

fn main() {}