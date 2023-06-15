use halo2_proofs::{
    arithmetic::Field,
    circuit::{AssignedCell, Layouter, SimpleFloorPlanner, Value},
    plonk::*,
    poly::Rotation,
};
use std::marker::PhantomData;

#[derive(Debug, Clone)]
struct CollatzConfig {
    pub advice: [Column<Advice>; 3],
    pub constant: Column<Fixed>,
    pub instance: Column<Instance>,
    pub s_mul: Selector,
    pub s_add: Selector,
}

#[derive(Debug, Clone)]
struct CollatzChip<F: Field> {
    config: CollatzConfig,
    _marker: PhantomData<F>,
}

#[derive(Clone)]
struct Number<F: Field>(AssignedCell<F, F>);

impl<F: Field> CollatzChip<F> {
    pub fn construct(config: CollatzConfig) -> Self {
        Self {
            config,
            _marker: PhantomData,
        }
    }

    pub fn configure(meta: &mut ConstraintSystem<F>) -> CollatzConfig {
        let advice = [meta.advice_column(); 3];
        let col_constant = meta.fixed_column();
        let s_mul = meta.selector();
        let s_add = meta.selector();
        let col_instance = meta.instance_column();

        for col in advice.into_iter() {
            meta.enable_equality(col);
        }
        meta.enable_equality(col_instance);
        meta.enable_equality(col_constant);
        meta.enable_constant(col_constant);

        // meta.create_gate("check divide", |meta| {
        // let lhs = meta.query_advice(advice[0], Rotation::cur());
        // let rhs = meta.query_advice(advice[1], Rotation::cur());
        // let out = meta.query_advice(advice[2], Rotation::cur());
        //
        // let s = meta.query_selector(s_mul);
        //
        // println!("{:?} * {:?} = {:?}", lhs, rhs, out);
        //
        // vec![s * (lhs * rhs - out)]
        // });

        // meta.create_gate("check add", |meta| {
        // let lhs = meta.query_advice(advice[0], Rotation::cur());
        // let rhs = meta.query_advice(advice[1], Rotation::cur());
        // let out = meta.query_advice(advice[2], Rotation::cur());
        //
        // let s = meta.query_selector(s_add);
        // println!("{:?} + {:?} = {:?}", lhs, rhs, out);
        //
        // vec![s * (lhs + rhs - out)]
        // });

        CollatzConfig {
            advice: advice,
            constant: col_constant,
            instance: col_instance,
            s_mul,
            s_add,
        }
    }

    fn load_constant(
        &self,
        mut layouter: impl Layouter<F>,
        constant: F,
    ) -> Result<Number<F>, Error> {
        let config = &self.config;

        layouter.assign_region(
            || "load constant",
            |mut region| {
                region
                    .assign_advice_from_constant(|| "constant value", config.advice[0], 0, constant)
                    .map(Number)
            },
        )
    }

    fn load_private(
        &self,
        mut layouter: impl Layouter<F>,
        value: Value<F>,
    ) -> Result<Number<F>, Error> {
        let config = &self.config;

        layouter.assign_region(
            || "load constant",
            |mut region| {
                region
                    .assign_advice(|| "private value", config.advice[0], 0, || value)
                    .map(Number)
            },
        )
    }

    fn mul(
        &self,
        mut layouter: impl Layouter<F>,
        a: Number<F>,
        b: Number<F>,
    ) -> Result<Number<F>, Error> {
        layouter.assign_region(
            || "mul",
            |mut region| {
                self.config.s_mul.enable(&mut region, 0)?;
                a.0.copy_advice(|| "lhs", &mut region, self.config.advice[0], 0)?;
                b.0.copy_advice(|| "rhs", &mut region, self.config.advice[1], 0)?;

                let value = a.0.value().copied() * b.0.value();

                println!(
                    "Sending: {:?} * {:?} = {:?}",
                    a.0.value().copied(),
                    b.0.value(),
                    value
                );

                region
                    .assign_advice(|| "lhs * rhs", self.config.advice[2], 0, || value)
                    .map(Number)
            },
        )
    }

    fn add(
        &self,
        mut layouter: impl Layouter<F>,
        a: Number<F>,
        b: Number<F>,
    ) -> Result<Number<F>, Error> {
        layouter.assign_region(
            || "add",
            |mut region| {
                self.config.s_add.enable(&mut region, 0)?;

                a.0.copy_advice(|| "lhs", &mut region, self.config.advice[0], 0)?;
                b.0.copy_advice(|| "rhs", &mut region, self.config.advice[1], 0)?;

                let value = a.0.value().copied() + b.0.value();

                region
                    .assign_advice(|| "lhs + rhs", self.config.advice[2], 0, || value)
                    .map(Number)
            },
        )
    }

    fn expose_public(
        &self,
        mut layouter: impl Layouter<F>,
        num: Number<F>,
        row: usize,
    ) -> Result<(), Error> {
        layouter.constrain_instance(num.0.cell(), self.config.instance, row)
    }
}

#[derive(Default)]
struct CollatzCircuit<F: Field> {
    n: Value<F>,
    div: F,
    mul: F,
    add: F,
}

impl<F: Field> Circuit<F> for CollatzCircuit<F> {
    type Config = CollatzConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        CollatzChip::configure(meta)
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl halo2_proofs::circuit::Layouter<F>,
    ) -> Result<(), Error> {
        let collatz_chip = CollatzChip::<F>::construct(config);

        let n = collatz_chip.load_private(layouter.namespace(|| "load a"), self.n)?;
        let div = collatz_chip.load_constant(layouter.namespace(|| "load constant"), self.div)?;
        let mul = collatz_chip.load_constant(layouter.namespace(|| "load constant"), self.mul)?;
        let add = collatz_chip.load_constant(layouter.namespace(|| "load constant"), self.add)?;

        let mut ndiv = collatz_chip.mul(layouter.namespace(|| "n * div"), n, div)?;
        ndiv = collatz_chip.mul(layouter.namespace(|| "n * div"), ndiv, mul)?;
        ndiv = collatz_chip.mul(layouter.namespace(|| "n * div"), ndiv, add)?;
        collatz_chip.expose_public(layouter.namespace(|| "expose ndiv"), ndiv, 0)
    }
}

fn main() {
    use halo2_proofs::{dev::MockProver, pasta::Fp};

    let n = Fp::from(5);
    let div = Fp::from(2);
    let mul = Fp::from(3);
    let add = Fp::from(1);

    let res = n * div;

    let circuit = CollatzCircuit {
        n: Value::known(n),
        div,
        mul,
        add,
    };

    let mut public_inputs = vec![res];

    let prover = MockProver::run(4, &circuit, vec![public_inputs.clone()]).unwrap();
    assert_eq!(prover.verify(), Ok(()));
}
