// Generate Proof

mod arithmetic_circuit;
use crate::arithmetic_circuit::arithmetic_circuit::{
    draw_circuit, ArithmeticChip, ArithmeticCircuit,
};
use halo2_proofs::{
    circuit::Value,
    dev::CircuitGates,
    pasta::{pallas, Fp},
};

// #[cfg(not(target_family = "wasm"))]
// fn main() {
//     let k = 4;
//     let constant = Fp::from(7);
//     let x = Fp::from(6);
//     let y = Fp::from(9);
//     let z = Fp::from(36 * 81 + 7);

//     let circuit: ArithmeticCircuit<Fp> = ArithmeticCircuit {
//         x: Value::known(x),
//         y: Value::known(y),
//         constant: constant,
//     };

//     draw_circuit(k, &circuit);
// }

fn main() {
    // Prepare the circuit you want to render.
    // You don't need to include any witness variables.
    let k = 4;
    let constant = Fp::from(7);
    let x = Fp::from(6);
    let y = Fp::from(9);
    let circuit: ArithmeticCircuit<Fp> = ArithmeticCircuit {
        x: Value::known(x),
        y: Value::known(y),
        constant: constant,
    };

    // Create the area you want to draw on.
    // Use SVGBackend if you want to render to .svg instead.
    use plotters::prelude::*;
    let root = BitMapBackend::new("layout.png", (1024, 768)).into_drawing_area();
    root.fill(&WHITE).unwrap();
    let root = root
        .titled("Arithmetic Circuit Layout", ("sans-serif", 30))
        .unwrap();

    // halo2_proofs::dev::CircuitLayout::default()
    //     // You can optionally render only a section of the circuit.
    //     // You can hide labels, which can be useful with smaller areas.
    //     .show_labels(true)
    //     // Render the circuit onto your area!
    //     // The first argument is the size parameter for the circuit.
    //     .render(5, &circuit, &root)
    //     .unwrap();

    halo2_proofs::dev::CircuitLayout::default()
        // .show_equality_constraints(true)
        .view_width(0..2)
        .view_height(0..16)
        .show_labels(true)
        .render(5, &circuit, &root)
        .unwrap();

    // Generate the DOT graph string.
    let dot_string = halo2_proofs::dev::circuit_dot_graph(&circuit);
    println!("GRAPH: {}", dot_string);
    let gates = CircuitGates::collect::<pallas::Base, ArithmeticCircuit<Fp>>();
    println!("{}", gates);
}
