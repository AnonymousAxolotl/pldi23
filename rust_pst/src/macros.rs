#[macro_export]
macro_rules! chans {
    () => { () };
    (...$rest:expr) => { $rest };
    ($i:ident) => { ($i, ()) };
    ($i:ident, $($tok:tt)*) => { ($i, $crate::chans![$($tok)*]) };
    ($a:expr) => { ($a, ()) };
    ($a:expr, $($tok:tt)*) => {
        ($a, $crate::chans![$($tok)*])
    };
}

#[macro_export]
macro_rules! Chans {
    () => { () };
    (...) => { _ };
    ($A:ty) => { ($crate::chan::Chan<$A>, ()) };
    ($A:ty, $($tok:tt)*) => {
        ($crate::chan::Chan<$A>, $crate::Chans![$($tok)*])
    };
}


#[macro_export]
macro_rules! Protos {
    () => { () };
    (...) => { _ };
    ($A:ty) => { ($A, ()) };
    ($A:ty,) => { ($A, ()) };
    ($A:ty, $($tok:tt)*) => {
        ($A, $crate::Protos![$($tok)*])
    };

    ($A:ty) => { ($A, ()) };
    ($A:ty;) => { ($A, ()) };
    ($A:ty; $($tok:tt)*) => {
        ($A, $crate::Protos![$($tok)*])
    };
}

#[macro_export]
macro_rules! Task {
    [$N:ty; $S:ty; $($tok:tt)*] => { impl $crate::chan::Task<$N, $crate::Protos![$($tok)*], $S> };
}

#[macro_export]
macro_rules! PrdT {
    [$PRD:ty] => { Prd<$PRD, P, _> };
}

#[macro_export]
macro_rules! spawn_task {
    ($($P:ty),+; $($tok:tt)*) => { spawn::<_, $crate::Protos!($($P,)*), _, _, _, _>($($tok)*)  };
}

#[macro_export]
macro_rules! task {
    (_; $($c:ident),* : $($ct:ty),* => $lambda:expr) => {
        move |chans![$($c),*]: $crate::Chans![$($ct),*], _: _| { $lambda }
    };
    ($s:ident; $($c:ident),* : $($ct:ty),* => $lambda:expr) => {
        move |chans![$($c),*]: $crate::Chans![$($ct),*], $s : _| { $lambda }
    };
    ($s:ident : $S:ty; $($c:ident),* : $($ct:ty),* => $lambda:expr) => {
        move |chans![$($c),*]: $crate::Chans![$($ct),*], $s : $S| { $lambda }
    };
    (mut $s:ident; $($c:ident),* : $($ct:ty),* => $lambda:expr) => {
        move |chans![$($c),*]: $crate::Chans![$($ct),*], mut $s : _| { $lambda }
    };
    (mut $s:ident : $S:ty; $($c:ident),* : $($ct:ty),* => $lambda:expr) => {
        move |chans![$($c),*]: $crate::Chans![$($ct),*], mut $s : $S| { $lambda }
    };
}

#[macro_export]
macro_rules! Compat {
    [$L: ty, $R: ty] => {
    $crate::session::CompatProofExpand
    <
        /* Number of times to expand L => Period of right */
        <$R as $crate::session::HasPrd>::PRD,
        /* Number of times to expand R => Period of left */
        <$L as $crate::session::HasPrd>::PRD,
        
        /* Period of L */
        <$L as $crate::session::HasPrd>::PRD,
        /* Period of R */
        <$R as $crate::session::HasPrd>::PRD,
        
        /* Thing to expand on left => body of left */
        <$L as $crate::session::HasPrd>::BODY,
        /* Thing to expand on right => body of right */
        <$R as $crate::session::HasPrd>::BODY,
        
        /* Proof of expansion on left */
        <<$R as $crate::session::HasPrd>::PRD as $crate::session::CanExpand<<$L as $crate::session::HasPrd>::BODY>>::EXP,
        /* Proof of expansion on right */
        <<$L as $crate::session::HasPrd>::PRD as $crate::session::CanExpand<<$R as $crate::session::HasPrd>::BODY>>::EXP,
        
        /* Proof of duality up to equivalence of substitution */
        $crate::session::EqDualCanon<
            /* Expanded left */
            <<<$R as $crate::session::HasPrd>::PRD as $crate::session::CanExpand<<$L as $crate::session::HasPrd>::BODY>>::EXP as $crate::session::Expansion<<$R as $crate::session::HasPrd>::PRD, <$L as $crate::session::HasPrd>::BODY>>::EXPANDED,
            /* Expanded right */
            <<<$L as $crate::session::HasPrd>::PRD as $crate::session::CanExpand<<$R as $crate::session::HasPrd>::BODY>>::EXP as $crate::session::Expansion<<$L as $crate::session::HasPrd>::PRD, <$R as $crate::session::HasPrd>::BODY>>::EXPANDED,
            /* Expansions should be dual */
            $crate::session::EqDualRefl<_>
        >
    >
    };
}

#[macro_export]
macro_rules! new_chan {
    () => {
        new_chan!(_, _)
    };
    ($L:ty, $R:ty) => {
        $crate::chan::Chan::<$L>::new::<$R, $crate::Compat![$L, $R]>()
    };
}