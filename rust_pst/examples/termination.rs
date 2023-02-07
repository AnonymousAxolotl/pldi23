use rust_pst::chan::Either;
use rust_pst::{new_chan, Task, task, chans, spawn};
use rust_pst::nat::ONE;
use rust_pst::session::{Send, Recv, Prd, P, Offer, Select, End};
use rust_pst::chan::TaskRet::{Continue, Terminate};

use std::thread;
use std::time::Duration;

type Selector = Select<Send<f32, P>, End>;
type Offerer = Offer<Recv<f32, P>, End>;

fn selector() -> Task![ONE; u32; Prd<ONE, P, Selector>]
{
    task![n; c : Selector => {
        if n > 0 {
            let ((), c) = c.left();
            let ((), c) = c.send(3.14159);
            thread::sleep(Duration::from_millis(250));
            Continue(chans![c], n-1)
        } else {
            let ((), c) = c.right();
            Terminate(chans![c])
        }
    }]
}

fn offerer() -> Task![ONE; (); Prd<ONE, P, Offerer>] 
{
    task![_; c : Offerer => { 
        let ((), e) = c.offer();
        match e {
            Either::Left(c) => {
                let (v, c) = c.recv();
                println!("{:?}", v);
                Continue(chans![c], ())
            },
            Either::Right(c) => Terminate(chans![c])
        }
    }]
}


fn main() {

    let (ka1, ka2)= new_chan!(Prd<ONE, P, Selector>, Prd<ONE, P, Offerer>);

    let  p1 = spawn(selector(), || chans!(ka1), || 10);
    let _p3 = spawn(offerer(),  || chans!(ka2), || ());

    let _ = p1.join();

}