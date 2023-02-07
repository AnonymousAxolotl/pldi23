use std::thread;
use std::time::Duration;
use std::fmt::Debug;
use std::marker::Send as RustSend;

use rust_pst::chan::{RecvAll, Chan};
use rust_pst::{Task, task, chans, new_chan, spawn};
use rust_pst::nat::{Nat, NonZero, ONE, FOUR};
use rust_pst::session::{Send, Recv, P, Prd, CanExpand, Expansion, HasCanon};
use rust_pst::chan::TaskRet::Continue;


use rust_pst::ppg::Biquad;

fn countf<N>() -> Task![N; f32; Prd<N, P, Send<f32, P>>] 
where
    N: Nat + NonZero,
{
    task![v; c : Send<f32, P> => { 
        thread::sleep(Duration::from_millis(250));
        let ((), c) = c.send(v);
        Continue(chans![c], v + 1.0)
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

fn task_decim_iir<N, R>() -> Task![N; Biquad; Prd<N, P, <<< R as CanExpand<Recv<f32, P>> >::EXP as Expansion<R, Recv<f32, P>>>::EXPANDED as HasCanon>::CANON >, Prd<N, P, Send<f32, P>>]
where
    N: Nat + NonZero,
    R: Nat + NonZero + CanExpand<Recv<f32, P>>,
    Chan<<<< R as CanExpand<Recv<f32, P>> >::EXP as Expansion<R, Recv<f32, P>>>::EXPANDED as HasCanon>::CANON > : RecvAll<f32, P>, <<R as CanExpand<Recv<f32, P>>>::EXP as Expansion<R, Recv<f32, P>>>::EXPANDED: HasCanon
{
    task![mut iir : Biquad; c1, c2 : <<< R as CanExpand<Recv<f32, P>> >::EXP as Expansion<R, Recv<f32, P>>>::EXPANDED as HasCanon>::CANON , Send<f32, P> => {
        let mut y = 0.0;
        
        let (v, p) = c1.recv_all();
        for x in v {
            y = iir.step(x);
        }
        let ((), c2) = c2.send(y);
        Continue(chans![p, c2], iir)
    }]
}

// But 2 : 4 also works, because rates still match
type ProdPrd = ONE;
type Decim = FOUR;
type ConsPrd = FOUR;

// See examples/rate12.rs
type ProdOut = Prd<ProdPrd, P, Send<f32, P>>;
type LPFIn   = Prd<ConsPrd, P, Recv<f32, Recv<f32, Recv<f32, Recv<f32, P>>>>>;
type LPFOut  = Prd<ConsPrd, P, Send<f32, P>>;
type ConsIn  = Prd<ConsPrd, P, Recv<f32, P>>;

fn main() {

    let (ka1, ka2)= new_chan!(ProdOut, LPFIn);
    let (kb1, kb2)= new_chan!(LPFOut, ConsIn);

    let  p1 = spawn(countf(),                       || chans!(ka1), || 0.0);
    let _p2 = spawn(task_decim_iir::<ConsPrd, Decim>(), || chans!(ka2, kb1), ||
        Biquad::new(
            [0.11595249, 0.23190498, 0.11595249], 
            [-0.72168143, 0.18549138]
    ));
    let _p3 = spawn(consume(),                      || chans!(kb2), || ());

    let _ = p1.join();

}