use rust_pst::{new_chan, Task, task, chans, spawn};
use rust_pst::nat::{ONE, Nat, NonZero};
use rust_pst::session::{Send, Recv, Prd, P};
use rust_pst::chan::TaskRet::Continue;
use rust_pst::ppg::{Biquad, AGC, PPG};

use std::fs::File;
use std::io::{BufReader, BufRead};
use std::time::Duration;
use std::thread;

fn task_adc(data: Vec<f32>) -> Task![ONE; usize; Prd<ONE, P, Send<f32, P>>]
{
    task![idx; c : Send<f32, P> => { 
        thread::sleep(Duration::from_millis(5));
        let ((), c) = c.send(data[idx]);
        Continue(chans![c], (idx+1) % data.len())
    }]
}

fn task_iir<N: Nat + NonZero>() -> Task![N; Biquad; Prd<N, P, Recv<f32, P>>, Prd<N, P, Send<f32, P>>]
{
    task![mut iir : Biquad; c1, c2 : Recv<f32, P>, Send<f32, P> => {
        let (x, c1) = c1.recv();
        let y = iir.step(x);
        let ((), c2) = c2.send(y);
        Continue(chans![c1, c2], iir)
    }]
}

fn task_agc() -> Task![ONE; AGC; Prd<ONE, P, Recv<f32, P>>, Prd<ONE, P, Send<f32, P>>]
{
    task![mut agc : AGC; c1, c2 : Recv<f32, P>, Send<f32, P> => {
        let (x, c1) = c1.recv();
        let y = agc.step(x);
        let ((), c2) = c2.send(y);
        Continue(chans![c1, c2], agc)
    }]
}

fn task_ppg() -> Task![ONE; PPG; Prd<ONE, P, Recv<f32, P>>]
{
    task![mut ppg : PPG; c : Recv<f32, P> => {
        let (x, c) = c.recv();
        ppg.step(x);
        Continue(chans![c], ppg)
    }]
}
// R_1 = \periodic[t]{1}{\nodeadline+1}{?\texttt{f32}.t} = \omega_{1} t. ?f32. t
type RX1F = Prd<ONE, P, Recv<f32, P>>;

// T_1 = \periodic[t]{1}{\nodeadline+1}{!\texttt{f32}.t} = \omega_{1} t. !f32. t
type TX1F = Prd<ONE, P, Send<f32, P>>;

fn main() -> std::io::Result<()> {
    // Parse the input data
    let txt = BufReader::new(File::open("ppg24.txt")?);
    let data: Vec<f32> = txt.lines().filter_map(Result::ok).map(|l| l.parse::<f32>().unwrap()).collect();

    // Create the channels
    let (ka1, ka2) = new_chan!(TX1F, RX1F);
    let (kb1, kb2) = new_chan!(TX1F, RX1F);
    let (kc1, kc2) = new_chan!(TX1F, RX1F);
    let (kd1, kd2) = new_chan!(TX1F, RX1F);

    // Spawn the tasks
    let  adc = spawn(task_adc(data), || chans!(ka1), || 0);
    let _hpf = spawn(task_iir::<ONE>(), || chans!(ka2, kb1), || Biquad::new([0.87033078, -1.74066156, 0.87033078], [-1.72377617, 0.75754694]));
    let _agc = spawn(task_agc(), || chans!(kb2, kc1), || AGC::new(400.0, 0.971, 2.0));
    let _lpf = spawn(task_iir(), || chans!(kc2, kd1), || Biquad::new([0.11595249, 0.23190498, 0.11595249], [-0.72168143, 0.18549138]));
    let _ppg = spawn(task_ppg(), || chans!(kd2), || PPG::new());

    // Wait forever
    let _ = adc.join();

    Ok(())
}
