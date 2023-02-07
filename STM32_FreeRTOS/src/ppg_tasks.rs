use crate::{Task, task, chans};
use crate::nat::{TWO, Nat, NonZero};
use crate::session::{Send, Recv, Prd, P};
use crate::chan::TaskRet::Continue;
use crate::ppg::{Biquad, AGC, PPG};

pub fn task_decim() -> Task![TWO; (); Prd<TWO, P, Recv<Option<f32>, Recv<Option<f32>, P>>>, Prd<TWO, P, Send<Option<f32>, P>>]
where
{
    task![_; c1, c2 : Recv<Option<f32>, Recv<Option<f32>, P>>, Send<Option<f32>, P> => {
        let (_, c1) = c1.recv();
        let (v, c1) = c1.recv();

        let ((), c2) = c2.send(v);
        Continue(chans![c1, c2], ())
    }]
}

pub fn task_iir<N>() -> Task![N; Biquad; Prd<N, P, Recv<Option<f32>, P>>, Prd<N, P, Send<Option<f32>, P>>]
where
    N: Nat + NonZero + 'static
{
    task![mut iir : Biquad; c1, c2 : Recv<Option<f32>, P>, Send<Option<f32>, P> => {
        let (x, c1) = c1.recv();
        let y = match x {
            Some(x) => Some(iir.step(x)),
            None => { iir.reset(); None }
        };
        let ((), c2) = c2.send(y);
        Continue(chans![c1, c2], iir)
    }]
}

pub fn task_agc<N>() -> Task![N; AGC; Prd<N, P, Recv<Option<f32>, P>>, Prd<N, P, Send<Option<f32>, P>>]
where
    N: Nat + NonZero + 'static
{
    task![mut agc : AGC; c1, c2 : Recv<Option<f32>, P>, Send<Option<f32>, P> => {
        let (x, c1) = c1.recv();
        let y = match x {
            Some(x) => Some(agc.step(x)),
            None => { agc.reset(); None}
        };
        let ((), c2) = c2.send(y);
        Continue(chans![c1, c2], agc)
    }]
}

pub fn task_ppg<N>() -> Task![N; (PPG, f32); Prd<N, P, Recv<Option<f32>, P>>, Prd<N, P, Send<(bool, f32, bool), P>>]
where
    N: Nat + NonZero + 'static
{
    task![state : (PPG, f32); ci, co : Recv<Option<f32>, P>, Send<(bool, f32, bool), P> => {
        let (mut ppg, last) = state;
        let (x, ci) = ci.recv();

        let co = match x {
            Some(x) => {
                let hr = ppg.step(x);
                if hr > 0.0 {
                    // rprintln!("Heart Rate = {:.0} bpm", hr)
                } else {
                    // rprintln!("No Reading -- Adjust Finger")
                }        
                let beat = x + last < 5.0;
                // rprintln!("{:3.2}", x);
                let ((), co) = co.send((true, hr, beat));
                co
            },
            None => {
                ppg.reset();
                // rprintln!("No Finger Detected");
                let ((), co) = co.send((false, -1.0, false));
                co
            }
        };

        Continue(chans![ci, co], (ppg, x.unwrap_or(0.0)))
    }]
}
