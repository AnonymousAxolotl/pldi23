use std::iter::zip;


pub struct Biquad {
    b: [f32; 3],
    a: [f32; 2],
    v1: f32, v2: f32,
}

impl Biquad {
    pub fn new(b: [f32; 3], a: [f32; 2]) -> Self {
        Self { b, a, v1: 0.0, v2: 0.0 }
    }

    pub fn step(&mut self, x: f32) -> f32 {
        let v = x - (self.a[0] * self.v1) - (self.a[1] * self.v2);
        let y = (self.b[0] * v) + (self.b[1] * self.v1) + (self.b[2] * self.v2);

        self.v2 = self.v1;
        self.v1 = v;
        
        y
    }
}

pub struct AGC {
    peak: f32,
    decay: f32,
    boost: f32,
    thresh: f32,
}

impl AGC {
    pub fn new(start: f32, decay: f32, thresh: f32) -> Self {
        Self {
            peak: start,
            decay,
            boost: decay.recip(),
            thresh
        }
    }

    pub fn step(&mut self, x: f32) -> f32 {
        self.peak *= if x.abs() > self.peak { self.boost } else { self.decay };
        if x.abs() > (self.peak * self.thresh) { 0.0 } else { 100.0 * x / (2.0 * self.peak) }
    }
}

pub struct PPG {
    buf: [f32; 144],
    hr: f32,
    idx: usize
}

impl PPG {
    pub fn new() -> Self {
        Self {
            buf: [0.0; 144],
            hr: -1.0,
            idx: 0,
        }
    }

    fn compare(d: &[f32], shift: usize) -> f32 {
        zip(&d[shift..], &d[..d.len()-shift])
            .map(|(a,b)| a - b)
            .map(|d| d*d).sum()
    }
    
    fn trough(d: &[f32], min: usize, max: usize) -> Option<usize> {
        let mut z2 = Self::compare(d, min-2);
        let mut z1 = Self::compare(d, min-1);
    
        for i in min..=max {
            let z = Self::compare(d, i);
            
            if z2 > z1 && z1 < z {
                return Some(i);
            }
    
            z2 = z1;
            z1 = z;
        }
    
        return None
    }

    fn process(&mut self) -> Option<f32> {
        let d = &self.buf;
        let t0 = Self::trough(d, 7, 48)?;
        let t1 = t0 * 2;
        let t1 = Self::trough(d, t1 - 5, t1 + 5)?;
        let t2 = (t1 * 3) / 2;
        let t2 = Self::trough(d, t2 - 5, t2 + 4)?;
        let t3 = (t2 * 4) / 3;
        let def: f32 = 3.0 / f32::from(u16::try_from(t2).unwrap());
        let f: f32 = Self::trough(d, t3 - 4, t3 + 4)
            .map_or(def, |t3| 4.0 / f32::from(u16::try_from(t3).unwrap()));

        return Some(60.0 * 24.0 * f)
    }

    pub fn step(&mut self, x: f32) {
        self.buf[self.idx] = x;
        self.idx += 1;

        if self.idx == self.buf.len() {
            self.hr = self.process().unwrap_or(-1.0);
            println!("HR = {}", self.hr);
            self.idx = 0;
        }
    }
}
