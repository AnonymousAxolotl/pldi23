use rust_pst::session::{Send, Recv, Prd, P};
use rust_pst::nat::{ONE, TWO};

type AdcOut = Prd<ONE, P, Send<f32, P>>;
type HpdIn  = Prd<ONE, P, Recv<f32, P>>;
type HdpOut = Prd<ONE, P, Send<f32, P>>;
type AgcIn  = Prd<ONE, P, Recv<f32, P>>;
type AgcOut = Prd<ONE, P, Send<f32, P>>;
type LpfIn  = Prd<TWO, P, Recv<f32, Recv<f32, P>>>;
type LpfOut = Prd<TWO, P, Send<f32, P>>;
type AutoIn = Prd<TWO, P, Recv<f32, P>>;

fn main() {}