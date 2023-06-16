// Generate Proof

fn generate_proof() {
    use halo2_proofs::dev::MockProver;
    use halo2_proofs::halo2curves::bn256::Fr as Fp;

    let k = 4;
    let constant = Fp::from(7);
    let x = Fp::from(6);
    let y = Fp::from(9);
    let z = Fp::from(36 * 81 + 7);

    let circuit: ArithmeticCircuit<Fp> = ArithmeticCircuit {
        x: Value::known(x),
        y: Value::known(y),
        constant: constant,
    };

    let mut public_inputs = vec![constant, z];
}
