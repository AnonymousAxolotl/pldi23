use rust_pst::chan::Either;
use rust_pst::{new_chan, Task, task, chans, spawn};
use rust_pst::nat::ONE;
use rust_pst::session::{Send, Recv, Prd, P, Offer, Select};
use rust_pst::chan::TaskRet::Continue;

use std::thread;
use std::time::Duration;

type Selector = Select<Send<i32, P>, Send<f32, P>>;
type Offerer = Offer<Recv<i32, P>, Recv<f32, P>>;

fn selector() -> Task![ONE; (); Prd<ONE, P, Selector>]
{
    task![_; c : Selector => {
        let ((), c) = c.right();
        let ((), c) = c.send(3.14159);
        thread::sleep(Duration::from_millis(250));
        Continue(chans![c], ())
    }]
}

fn offerer() -> Task![ONE; (); Prd<ONE, P, Offerer>] 
{
    task![_; c : Offerer => { 
        let ((), e) = c.offer();
        let c = match e {
            Either::Left(c) => {
                let (v, c) = c.recv();
                println!("{:?}", v);
                c
            },
            Either::Right(c) => {
                let (v, c) = c.recv();
                println!("{:?}", v);
                c
            },
        };
        Continue(chans![c], ())
    }]
}


fn main() {

    let (ka1, ka2)= new_chan!(Prd<ONE, P, Selector>, Prd<ONE, P, Offerer>);

    let  p1 = spawn(selector(), || chans!(ka1), || ());
    let _p3 = spawn(offerer(),  || chans!(ka2), || ());

    let _ = p1.join();

}