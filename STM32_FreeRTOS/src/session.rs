use core::marker::{PhantomData, Send as RustSend};
use crate::nat::{S, Z, Nat, NonZero, ONE};

pub trait Protocol: RustSend { type DUAL: Protocol; type NEXT: RustSend; }

pub struct Send<T: RustSend, K: Protocol> {
    t: PhantomData<T>,
    k: PhantomData<K>
}
impl<T: RustSend, K: Protocol> Protocol for Send<T, K> {
    type DUAL = Recv<T, K::DUAL>;
    type NEXT = T;
}
pub struct Recv<T: RustSend, K: Protocol> {
    t: PhantomData<T>,
    k: PhantomData<K>
}
impl<T: RustSend, K: Protocol> Protocol for Recv<T, K> {
    type DUAL = Send<T, K::DUAL>;
    type NEXT = T;
}

pub struct End { }
impl Protocol for End { type DUAL = End; type NEXT = (); }

pub struct Offer<L: Protocol, R: Protocol> {
    l: PhantomData<L>,
    r: PhantomData<R>
}

pub struct Select<L: Protocol, R: Protocol> {
    l: PhantomData<L>,
    r: PhantomData<R>
}

pub enum Label {
    Left,
    Right,
}

impl<L: Protocol, R: Protocol> Protocol for Offer<L, R> {
    type DUAL = Select<L::DUAL, R::DUAL>;
    type NEXT = Label;
}

impl<L: Protocol, R: Protocol> Protocol for Select<L, R> {
    type DUAL = Offer<L::DUAL, R::DUAL>;
    type NEXT = Label;
}

pub struct P { }
pub trait PVar: RustSend { }
impl PVar for P { }
impl<T: PVar> Protocol for T { type DUAL = Self; type NEXT = (); }

pub struct Rec<T: PVar, P: Protocol> {
    p: PhantomData<P>,
    t: PhantomData<T>
}

impl<T: PVar, P: Protocol> Protocol for Rec<T, P> {
    type DUAL = Rec<T, P::DUAL>;
    type NEXT = P::NEXT;
}

pub trait HasPrd {
    type PRD : Nat;
    type BODY : Protocol;
}

pub struct Prd<N: Nat, T: PVar, P: Protocol> {
    n: PhantomData<N>,
    p: PhantomData<P>,
    t: PhantomData<T>
}

impl<N: Nat, T: PVar, P: Protocol> Protocol for Prd<N, T, P> {
    type DUAL = Prd<N, T, P::DUAL>;
    type NEXT = P::NEXT;
}

impl<N: Nat, T: PVar, P: Protocol> HasPrd for  Prd<N, T, P> {
    type PRD = N;
    type BODY = P;
}

pub trait Compat<L: Protocol, R: Protocol> { }
pub struct CompatProofDual { }
impl<P: Protocol> Compat<P, P::DUAL> for CompatProofDual { }

pub struct CompatProofEqDual<L: Protocol, R: Protocol, W: EqDual<L, R>> { p: PhantomData<(L, R, W)> }
impl<L: Protocol, R: Protocol, W: EqDual<L, R>> Compat<L, R> for CompatProofEqDual<L, R, W> { }

pub struct CompatProofExpand<
    N: Nat + NonZero,
    M: Nat + NonZero,
    PL: Nat + NonZero,
    PR: Nat + NonZero,
    L: Protocol,
    R: Protocol,
    EL: Expansion<N, L>,
    ER: Expansion<M, R>,
    W: EqDual<EL::EXPANDED, ER::EXPANDED>
>
{ p: PhantomData<(N, M, PL, L, PR, R, EL, ER, W)> }

impl<N, M, PL, L, PR, R, EL, ER, W> Compat<Prd<PL, P, L>, Prd<PR, P, R>> for CompatProofExpand<N, M, PL, PR, L, R, EL, ER, W>
where
    N: Nat + NonZero,
    M: Nat + NonZero,
    PL: Nat + NonZero,
    PR: Nat + NonZero,
    L: Protocol,
    R: Protocol,
    EL: Expansion<N, L>,
    ER: Expansion<M, R>,
    W: EqDual<EL::EXPANDED, ER::EXPANDED>
{ }

// pub struct CompatProofExpandL<N, L, R, E, W> { p: PhantomData<(N, L, R, E, W)> }
// impl<L, R, E, W, N> Compat<L, R> for CompatProofExpandL<N, L, R, E, W>
// where
//     N: Nat,
//     L: Protocol,
//     R: Protocol,
//     E: Expansion<N, L>,
//     W: EqDual<E::EXPANDED, R>
// { }

// pub struct CompatProofExpandR<N, L, R, E, W> { p: PhantomData<(N, L, R, E, W)> }
// impl<L, R, E, W, N> Compat<L, R> for CompatProofExpandR<N, L, R, E, W>
// where
//     N: Nat,
//     L: Protocol,
//     R: Protocol,
//     E: Expansion<N, R>,
//     W: EqDual<L, E::EXPANDED>
// { }

pub struct SubP<R: Protocol, T: Protocol> {
    r: PhantomData<R>,
    b: PhantomData<T>
}

impl<R: Protocol, T: Protocol> Protocol for SubP<R, T> {
    type DUAL = SubP<R::DUAL, <T as Protocol>::DUAL>;
    type NEXT = <T as Protocol>::NEXT;
}

pub trait HasCanon: Protocol {
    type CANON : Protocol + HasCanon;
}

impl<T: RustSend, K: Protocol> HasCanon for Send<T, K> {
    type CANON = Self;
}

impl<T: RustSend, K: Protocol> HasCanon for Recv<T, K> {
    type CANON = Self;
}

impl<L: Protocol, R: Protocol> HasCanon for Offer<L, R> {
    type CANON = Self;
}

impl<L: Protocol, R: Protocol> HasCanon for Select<L, R> {
    type CANON = Self;
}

impl<N: Nat, T: Protocol, P: PVar> HasCanon for Prd<N, P, T>
{
    type CANON = Self;
}

impl<R: Protocol + HasCanon> HasCanon for SubP<R, P>
{
    type CANON = R;
}

impl<R: Protocol, K: Protocol, V: RustSend> HasCanon for SubP<R, Send<V, K>>
where
    SubP<R, K>: HasCanon
{
    type CANON = Send<V, <SubP<R, K> as HasCanon>::CANON>;
}

impl<R: Protocol, K: Protocol, V: RustSend> HasCanon for SubP<R, Recv<V, K>>
where
    SubP<R, K>: HasCanon
{
    type CANON = Recv<V, <SubP<R, K> as HasCanon>::CANON>;
}


impl<R: Protocol, LL: Protocol, LR: Protocol> HasCanon for SubP<R, Offer<LL, LR>>
where
    SubP<R, LL>: HasCanon,
    SubP<R, LR>: HasCanon,
{
    type CANON = Offer<<SubP<R, LL> as HasCanon>::CANON, <SubP<R, LR> as HasCanon>::CANON>;
}

impl<R: Protocol, LL: Protocol, LR: Protocol> HasCanon for SubP<R, Select<LL, LR>>
where
    SubP<R, LL>: HasCanon,
    SubP<R, LR>: HasCanon,
{
    type CANON = Select<<SubP<R, LL> as HasCanon>::CANON, <SubP<R, LR> as HasCanon>::CANON>;
}




impl<R: Protocol, N: Nat, T: Protocol + HasCanon, P: PVar> HasCanon for SubP<R, Prd<N, P, T>>
where
    SubP<R, T>: HasCanon
{
    type CANON = Prd<N, P, <SubP<R, T> as HasCanon>::CANON>;
}

impl<R: Protocol, S: Protocol, T: Protocol + HasCanon> HasCanon for SubP<R, SubP<S, T>>
where
    SubP<R, SubP<S, T>>: Protocol,
    SubP<S, T>: HasCanon,
    SubP<R, <SubP<S, T> as HasCanon>::CANON>: HasCanon,
    <SubP<R, <SubP<S, T> as HasCanon>::CANON> as HasCanon>::CANON: HasCanon
{
    type CANON = <SubP<R, <SubP<S, T> as HasCanon>::CANON> as HasCanon>::CANON;
}

pub trait EqDual<A: Protocol, B: Protocol> { }

pub struct EqDualRefl<T: Protocol> { t: PhantomData<T> }
impl<T: Protocol> EqDual<T, T::DUAL> for EqDualRefl<T> { }

pub struct EqDualCanon<A: Protocol + HasCanon, B: Protocol + HasCanon, W: EqDual<A::CANON, B::CANON>> { t: PhantomData<(A, B, W)> }
impl<
    A: Protocol + HasCanon,
    B: Protocol + HasCanon,
    W: EqDual<A::CANON, B::CANON>
> EqDual<A, B> for EqDualCanon<A, B, W> { }


pub trait Expansion<N: Nat, T: Protocol> {
    type EXPANDED: Protocol;
}

pub struct ExpandOne { }
impl<T: Protocol> Expansion<ONE, T> for ExpandOne {
    type EXPANDED = T;
}

pub struct ExpandN<N: Nat + NonZero, T: Protocol, E: Expansion<N, T>> { p: PhantomData<(N, T, E)> }

impl<N: Nat, T: Protocol, E: Expansion<S<N>, T> > Expansion<S<S<N>>, T> for ExpandN<S<N>, T, E>
where
    SubP<T, E::EXPANDED>: Protocol
{
    type EXPANDED = SubP<T, E::EXPANDED>;
}

pub trait CanExpand<T: Protocol>: Nat {
    type EXP: Expansion<Self, T>;
}

impl<T: Protocol> CanExpand<T> for S<Z> {
    type EXP = ExpandOne;
}

impl<N: Nat + NonZero + CanExpand<T>, T: Protocol> CanExpand<T> for S<N>
where
    ExpandN<N, T, <N as CanExpand<T>>::EXP>: Expansion<S<N>, T>
{
    type EXP = ExpandN<N, T, <N as CanExpand<T>>::EXP>;
}

