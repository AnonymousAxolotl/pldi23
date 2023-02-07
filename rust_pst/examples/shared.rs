
use rust_pst::ppg::{Biquad, AGC, PPG};

use std::fs::File;
use std::io::{BufReader, BufRead};

fn main() -> std::io::Result<()> {

    let txt = BufReader::new(File::open("ppg24.txt")?);
    let data: Vec<f32> = txt.lines().filter_map(Result::ok).map(|l| l.parse::<f32>().unwrap()).collect();

    let mut hpf = Biquad::new([0.87033078, -1.74066156, 0.87033078], [-1.72377617, 0.75754694]);
    let mut agc = AGC::new(400.0, 0.971, 2.0);
    let mut lpf = Biquad::new([0.11595249, 0.23190498, 0.11595249], [-0.72168143, 0.18549138]);
    let mut ppg = PPG::new();

    for x in data {
        let x1 = hpf.step(x);
        let x2 = agc.step(x1);
        let x3 = lpf.step(x2);
        ppg.step(x3);
    }

    Ok(())
}