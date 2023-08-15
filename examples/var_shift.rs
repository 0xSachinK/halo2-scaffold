use std::env::var;
use clap::Parser;
use halo2_base::safe_types::RangeInstructions;
use halo2_scaffold::scaffold::cmd::Cli;
use halo2_base::gates::{GateChip, RangeChip, GateInstructions};
use halo2_scaffold::scaffold::run;
use halo2_base::utils::ScalarField;
use halo2_base::AssignedValue;
use halo2_base::{
    Context,
    QuantumCell::{Constant, Existing, Witness},
};
use serde::{Deserialize, Serialize};

const ARR_LEN: usize = 10;
const NUM_BITS: usize = 10;


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CircuitInput {
    pub arr: Vec<String>, // Vector of String of length 1000
    pub start: String, // [0, 1000)
    pub end: String, // [0, 1000)
    // start <= end
}

// public inputs:
// * An array `arr` of length 1000
// * `start`, an index guaranteed to be in `[0, 1000)`
// * `end`, an index guaranteed to be in `[0, 1000)`
// * It is also known that `start <= end`

// public outputs:
// * An array `out` of length 1000 such that
//   * the first `end - start` entries of `out` are the subarray `arr[start:end]`
//   * all other entries of `out` are 0.
fn some_algorithm_in_zk<F: ScalarField>(
    ctx: &mut Context<F>,
    input: CircuitInput,
    make_public: &mut Vec<AssignedValue<F>>
) {
    
    let mut input_arr: Vec<AssignedValue<F>> = Vec::with_capacity(ARR_LEN);

    // // Desrialize input values and insert them into the table
    for i in 0..ARR_LEN {
        let temp = F::from_str_vartime(&input.arr[i]).expect(
            "deserialize field element should not fail"
        );
        let temp = ctx.load_witness(temp); 
        // Make input values public
        input_arr.push(temp);
        make_public.push(temp);
    }

    let start = F::from_str_vartime(&input.start).expect("deserialize field element should not fail");
    let start = ctx.load_witness(start);
    make_public.push(start);

    let end = F::from_str_vartime(&input.end).expect("deserialize input should not fail");
    let end = ctx.load_witness(end);
    make_public.push(end);

    let lookup_bits =
        var("LOOKUP_BITS").unwrap_or_else(|_| panic!("LOOKUP_BITS not set")).parse().unwrap();
    
    // Initialize gate
    let gate = RangeChip::default(lookup_bits);

    // Guaranteed that start <= end. So need not check.

    // Shift input arr left by start indices
    let arr = var_shift_left(ctx, &gate, input_arr, start);

    // Take first (end - start) elements of the array
    let k = gate.gate().sub(ctx, end, start);
    let out = var_sub_array_from_zero_index(
        ctx, 
        &gate, 
        arr, 
        k
    );

    // make output array public
    for i in 0..ARR_LEN {
        make_public.push(out[i]);
    }

    // Print values for debugging
    // print inputs
    println!("start: {:?}", start.value());
    println!("end: {:?}", end.value());
    // for i in 0..ARR_LEN {
    //     println!("input {}: {:?}", i, input_arr[i].value());
    // }
    // for i in 0..ARR_LEN {
    //     println!("inter {}: {:?}", i, arr[i].value());
    // }
    for i in 0..ARR_LEN {
        println!("out {}: {:?}", i, out[i].value());
    }


}

fn var_shift_left<F: ScalarField>(
    ctx: &mut Context<F>,
    gate: &RangeChip<F>,
    arr: Vec<AssignedValue<F>>,
    shift: AssignedValue<F>
) -> Vec<AssignedValue<F>> {

    // convert shift to bits
    let shift_bits = gate.gate().num_to_bits(ctx, shift, NUM_BITS);

    // initialize a mutable vector of dimensions [numBits][ArrLen]
    let mut temp: Vec<Vec<AssignedValue<F>>> = Vec::with_capacity(NUM_BITS);

    for j in 0..NUM_BITS {

        let mut temp_j: Vec<AssignedValue<F>> = Vec::with_capacity(ARR_LEN);

        for i in 0..ARR_LEN {
            let offset = (i + (1 << j)) % ARR_LEN;
            if j == 0 {
                // temp[j][i] = shift_bits[j] * (arr[offset] - arr[i]) + arr[i]
                let diff = gate.gate().sub(ctx, arr[offset], arr[i]);
                temp_j.push(gate.gate().mul_add(
                    ctx, 
                    diff, 
                    shift_bits[j], 
                    arr[i]
                ));
            } else {
                // temp[j][i] = shift_bits[j] * (temp[j - 1][offset] - temp[j - 1][i]) + temp[j - 1][i]
                let diff = gate.gate().sub(ctx, temp[j - 1][offset], temp[j - 1][i]);
                temp_j.push(gate.gate().mul_add(
                    ctx, 
                    diff,
                    shift_bits[j], 
                    temp[j - 1][i]
                ));
            }
        }

        temp.push(temp_j);
    }

    // return last row
    // TODO: Do I need to clone it?
    temp[NUM_BITS - 1].clone()
}


fn var_sub_array_from_zero_index<F: ScalarField>(
    ctx: &mut Context<F>,
    gate: &RangeChip<F>, // RangeChip contains methods for checking range and other useful comparators
    arr: Vec<AssignedValue<F>>,
    end: AssignedValue<F>
) -> Vec<AssignedValue<F>> {

    // Initialize a vector to store out
    let mut out: Vec<AssignedValue<F>>= Vec::with_capacity(ARR_LEN);

    for i in 0..ARR_LEN {
        let less_than = gate.is_less_than(
            ctx, 
            Constant(F::from(i as u64)), 
            end, 
            NUM_BITS
        );
        out.push(gate.gate().mul(ctx, less_than, arr[i]));
    }

    out
}


fn main() {
    env_logger::init();

    let args = Cli::parse();

    // run different zk commands based on the command line arguments
    run(some_algorithm_in_zk, args);

}





