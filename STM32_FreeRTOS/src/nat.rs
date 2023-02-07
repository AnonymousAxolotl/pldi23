#![allow(dead_code)]

use core::marker::PhantomData;

pub trait Nat: Send + Copy { 
    type SUCC;
    fn val() -> u32;
}

#[derive(Clone, Copy)]
pub struct Z { }

#[derive(Clone, Copy)]
pub struct S<N: Nat> { n : PhantomData<N> }

impl Nat for Z { type SUCC = S<Self>; fn val() -> u32 { 0 }}
impl<N: Nat> Nat for S<N> { type SUCC = S<Self>; fn val() -> u32 { 1+N::val() } }

pub trait Diff<N: Nat, M: Nat> { type R: Nat; }
impl<N: Nat> Diff<N, Z> for PhantomData<(N, Z)> { type R = N; }
impl<N: Nat, M: Nat> Diff<S<N>, S<M>> for PhantomData<(S<N>, S<M>)> where PhantomData<(N, M)>: Diff<N, M> { type R = <PhantomData<(N, M)> as Diff<N, M>>::R; }

pub trait Greater<N: Nat, M: Nat> : Diff<N, M> { }
impl<N: Nat, M: Nat, D: Diff<N, M>> Greater<N,M> for D { }

pub trait NonZero { }
impl<N: Nat> NonZero for S<N> { }

pub type ZERO = Z;
pub type ONE = S<ZERO>;
pub type TWO = S<ONE>;
pub type THREE = S<TWO>;
pub type FOUR = S<THREE>;
pub type FIVE = S<FOUR>;
pub type SIX = S<FIVE>;
pub type SEVEN = S<SIX>;
pub type EIGHT = S<SEVEN>;
pub type NINE = S<EIGHT>;
pub type TEN = S<NINE>;
pub type ELEVEN = S<TEN>;
pub type TWELVE = S<ELEVEN>;
pub type THIRTEEN = S<TWELVE>;
pub type FOURTEEN = S<THIRTEEN>;
pub type FIFTEEN = S<FOURTEEN>;
pub type SIXTEEN = S<FIFTEEN>;
pub type SEVENTEEN = S<SIXTEEN>;
pub type EIGHTEEN = S<SEVENTEEN>;
pub type NINETEEN = S<EIGHTEEN>;
pub type TWENTY = S<NINETEEN>;
