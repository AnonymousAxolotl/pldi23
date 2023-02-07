
use rust_pst::new_chan;
use rust_pst::nat::{TWO, THREE};
use rust_pst::session::{Send, Recv, Prd, P};

// P_2 = \periodic[t]{2}{\nodeadline+2}{!\texttt{int}. !\texttt{int}.t} = \omega_{2} t. !int. !int. t
type P2 = Prd<TWO, P, Send<i32, Send<i32, P>>>;

// C_3 = \periodic[t]{3}{\nodeadline+3}{?\texttt{int}. ?\texttt{int}. ?\texttt{int}.t} = \omega_{3} t. ?int. ?int. ?int. t
type C3 = Prd<THREE, P, Recv<i32, Recv<i32, Recv<i32, P>>>>;

fn main() {
    let _ = new_chan![P2, C3];
}