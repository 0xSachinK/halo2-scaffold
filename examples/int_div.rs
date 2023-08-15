use clap::Parser;
use halo2_scaffold::scaffold::cmd::Cli;
use halo2_base::gates::{GateChip, GateInstructions};
use halo2_scaffold::scaffold::run;
use halo2_base::utils::ScalarField;
use halo2_base::AssignedValue;
use halo2_base::{
    Context,
    QuantumCell::{Constant, Existing, Witness},
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CircuitInput {
    pub x: String, // field element, but easier to deserialize as a string
}

// public inputs:
// * A non-negative integer x, which is guaranteed to be at most 16-bits
// public outputs:
// * The non-negative integer (x / 32), where "/" represents integer division.
fn some_algorithm_in_zk<F: ScalarField>(
    ctx: &mut Context<F>,
    input: CircuitInput,
    make_public: &mut Vec<AssignedValue<F>>
) {

    let x = F::from_str_vartime(&input.x).expect("deserialize field element should not fail");

    // load x as witness into the table
    let x = ctx.load_witness(x);
    
    // By default, all witness values are private. We explicitly make x public.
    make_public.push(x);

    // Create a Gate chip that contains methods for basic arithmetic operations
    let gate = GateChip::<F>::default();

    // 2 ^ 16 = 65536. 0 < x < 65536
    // Output: x / 32. Integer division by 32 means right shifting the binary representation of x by 5 bits.
    
    let bits = gate.num_to_bits(ctx, x, 16);

    // Initialize a mutable vector of 16 - 5 = 11 elements
    let mut out: Vec<AssignedValue<F>> = Vec::with_capacity(11);

    out.push(gate.mul_add(ctx, bits[5], Constant(F::from(1)), Constant(F::from(0))));

    
    for i in 1..11 {
        let acc = gate.mul_add(ctx, bits[i + 5], Constant(gate.pow_of_two[i]), out[i - 1]);
        out.push(acc);
    }

    // out[10] is the output of the circuit
    make_public.push(out[10]);


    // print values
    println!("x: {:?}", x.value());
    for i in 0..16 {
        println!("bit {}: {:?}", i, bits[i].value());
    }
    for i in 0..11 {
        println!("out {}: {:?}", i, out[i].value());
    }
    println!("result: {:?}", out[10].value());
}


fn main() {
    env_logger::init();

    let args = Cli::parse();

    // run different zk commands based on the command line arguments
    run(some_algorithm_in_zk, args);

}





