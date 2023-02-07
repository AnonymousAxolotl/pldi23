#![allow(dead_code)]

use core::marker::{PhantomData, Send as RustSend};
use core::mem::transmute;

use freertos_rust::{TaskPriority, Task as RTOSTask};

use crate::nat::{Nat, S, NonZero};
use crate::pkt::Packet;
use crate::session::{Protocol, Send, Recv, Compat, PVar, Rec, Prd, Label, Offer, Select, End};

pub enum Either<L, R> {
    Left(L),
    Right(R),
}

pub enum TaskRet<C: Chans, V> {
    Continue(C, V),
    Terminate(C::DONE)
}

pub struct Chan<P: Protocol + 'static> {
    pkt: &'static Packet<P>,
    p: PhantomData<P>,
}

impl<L: Protocol + 'static> Chan<L> {
    pub fn new<R, W>(pkt: &'static mut Packet<L>) -> (Chan<L>, Chan<R>) 
    where
        R: Protocol,
        W: Compat<L, R>
    {
        let left = Chan { pkt, p: PhantomData };
        // SAFETY: We need a transmute here because rustc can't prove
        // Compat<L, R> implies L::NEXT == R::NEXT
        let right = Chan { pkt: unsafe { pkt.transmute_ref() }, p: PhantomData };

        (left, right)
    }
}

impl<T: RustSend, K: Protocol> Chan<Send<T, K>> {
    pub fn send(self, v: T) -> ((), Chan<K>) {
        let pkt : &Packet<T> = unsafe { self.pkt.transmute_ref() };
        pkt.send(v);
        pkt.reset();
        let chan = unsafe { transmute(self) };
        ((), chan)
    }
}

impl<T: RustSend, K: Protocol> Chan<Recv<T, K>> {
    pub fn recv(self) -> (T, Chan<K>) {
        let pkt : &Packet<T> = unsafe { self.pkt.transmute_ref() };
        let v = pkt.recv();
        let chan = unsafe { transmute(self) };
        (v, chan)
    }
}

impl<L: Protocol, R: Protocol> Chan<Offer<L, R>> {
    pub fn offer(self) -> ((), Either<Chan<L>, Chan<R>>) {
        let pkt : &Packet<Label> =  unsafe { self.pkt.transmute_ref() };
        let l = pkt.recv();
        
        let chan = match l {
            Label::Left => Either::Left(unsafe { transmute(self) }),
            Label::Right => Either::Right(unsafe { transmute(self) })
        };
        
        ((), chan)
    }
}

impl<L: Protocol, R: Protocol> Chan<Select<L, R>> {
    pub fn left(self) -> ((), Chan<L>) {
        let pkt : &Packet<Label> = unsafe { self.pkt.transmute_ref() };
        pkt.send(Label::Left);
        pkt.reset();
        let chan = unsafe { transmute(self) };
        ((), chan)
    }

    pub fn right(self) -> ((), Chan<R>) {
        let pkt : &Packet<Label> = unsafe { self.pkt.transmute_ref() };
        pkt.send(Label::Right);
        pkt.reset();
        let chan = unsafe { transmute(self) };
        ((), chan)
    }
}

// pub trait RecvAll<V: RustSend, P: PVar> {
//     fn recv_all(self) -> (Vec<V>, Chan<P>);
// }

// impl<V: RustSend, P: PVar> RecvAll<V, P> for Chan<P> {
//     fn recv_all(self) -> (Vec<V>, Chan<P>) {
//         (vec!(), self)
//     }
// }

// impl<V: RustSend, P: PVar, K: Protocol> RecvAll<V, P> for Chan<Recv<V, K>> 
// where Chan<K> : RecvAll<V, P>
// {
//     fn recv_all(self) -> (Vec<V>, Chan<P>) {
//         let (x, c) = self.recv();
//         let (mut v, p) = c.recv_all();
//         v.insert(0, x);
//         (v, p)
//     }
// }

impl<T: PVar, P: Protocol> Chan<Rec<T, P>> { }

impl<N: Nat, T: PVar, P: Protocol> Chan<Prd<N, T, P>> {
    fn enter(self) -> Chan<P> {
        unsafe { core::mem::transmute(self) }
    }

    fn call(c: Chan<T>) -> Self {
        unsafe { core::mem::transmute(c) }
    }
}

pub trait Chans: RustSend { 
    type DONE;
}
impl Chans for () { 
    type DONE =  ();
}
impl<P: Protocol, T: Chans> Chans for (Chan<P>, T) {
    type DONE = (Chan<End>, T::DONE);
}

pub trait PrdChans<N: Nat + NonZero> { 
    type CHANS : Chans; type BODS: Chans; type VARS: Chans;
    fn enter_all(c: Self::CHANS) -> Self::BODS;
    fn call_all(c: Self::VARS) -> Self::CHANS;
}
impl<N: Nat + NonZero> PrdChans<N> for () { 
    type CHANS = (); type BODS = (); type VARS = ();
    fn enter_all(_: Self::CHANS) -> Self::BODS { () }
    fn call_all(_: Self::VARS) -> Self::CHANS { () }
}
impl<N: Nat + NonZero + 'static, P: Protocol + 'static, T: PVar + 'static, TL: PrdChans<N>> PrdChans<N> for (Prd<N, T, P>, TL) { 
    type CHANS = (Chan<Prd<N, T, P>>, TL::CHANS);
    type BODS = (Chan<P>, TL::BODS);
    type VARS = (Chan<T>, TL::VARS);

    fn enter_all(c: Self::CHANS) -> Self::BODS {
        let (hd, tl) = c;
        (hd.enter(), TL::enter_all(tl))
    }

    fn call_all(c: Self::VARS) -> Self::CHANS {
        let (hd, tl) = c;
        (Chan::<Prd<N, T, P>>::call(hd), TL::call_all(tl))
    }
}

pub trait Task<N: Nat + NonZero, C: PrdChans<N> + RustSend, V>: Fn(C::BODS, V)->TaskRet<C::VARS, V> + RustSend + 'static {
    fn exec(&self, c: C::BODS, v: V) -> TaskRet<C::VARS, V>;
}

impl<N: Nat + NonZero, C: PrdChans<N> + RustSend, V, F: Fn(C::BODS, V)->TaskRet<C::VARS, V> + RustSend + 'static> Task<N, C, V> for F { 
    fn exec(&self, c: <C as PrdChans<N>>::BODS, v: V) ->  TaskRet<<C as PrdChans<N>>::VARS, V> {
        self(c, v)
    }
}

pub fn register<N,C,V,T,G,H>(name: &str, stack_size: u16, f: T, c: H, init: G) -> RTOSTask
where
    N: Nat,
    C: PrdChans<S<N>> + RustSend,
    T: Task<S<N>, C, V>,
    G: FnOnce() -> V + RustSend + 'static,
    H: FnOnce() -> C::CHANS + core::marker::Send + 'static,
{
    RTOSTask::new().name(name).stack_size(stack_size).priority(TaskPriority(2)).start(move |_| {
        let mut chans = c();
        let mut state = init();
        loop {
            match f.exec(C::enter_all(chans), state) {
                TaskRet::Continue(new_chans, new_state) => {
                    state = new_state;
                    chans = C::call_all(new_chans);
                },
                TaskRet::Terminate(_) => break
            }
        }
    }).unwrap()
}
