use arith::SimdField;
use circuit_std_rs::sha256::gf2::SHA256GF2;
use expander_compiler::frontend::*;
use rand::RngCore;
use serdes::ExpSerde;
use sha2::{Digest, Sha256};

// ref: https://github.com/PolyhedraZK/ExpanderCompilerCollection/blob/master/circuit-std-rs/tests/sha256_gf2.rs#L89-L137
const INPUT_LEN: usize = 1024 * 8; // input size in bits, must be a multiple of 8
const OUTPUT_LEN: usize = 256; // FIXED 256

declare_circuit!(SHA256Circuit {
    input: [Variable; INPUT_LEN],
    output: [PublicVariable; OUTPUT_LEN],
});

impl Define<GF2Config> for SHA256Circuit<Variable> {
    fn define<Builder: RootAPI<GF2Config>>(&self, api: &mut Builder) {
        let mut hasher = SHA256GF2::new();
        hasher.update(&self.input);
        let output = hasher.finalize(api);
        (0..OUTPUT_LEN).for_each(|i| api.assert_is_equal(output[i], self.output[i]));
    }
}

fn main() {
    assert!(INPUT_LEN % 8 == 0);
    let n_witnesses = SIMDField::<GF2Config>::PACK_SIZE;

    // prepare data
    let mut rng = rand::rng();
    let data = [rng.next_u32() as u8; INPUT_LEN / 8];
    let mut hash = Sha256::new();
    hash.update(data);
    let output = hash.finalize();

    // compile the circuit
    let compile_result = compile(&SHA256Circuit::default(), CompileOptions::default()).unwrap();

    // prepare assignment
    let mut assignment = SHA256Circuit::default();
    for i in 0..INPUT_LEN / 8 {
        for j in 0..8 {
            assignment.input[i * 8 + j] = (((data[i] >> (7 - j)) & 1) as u32).into();
        }
    }
    for i in 0..OUTPUT_LEN / 8 {
        for j in 0..8 {
            assignment.output[i * 8 + j] = (((output[i] >> (7 - j) as u32) & 1) as u32).into();
        }
    }

    let mut assignments: Vec<SHA256Circuit<GF2>> = vec![SHA256Circuit::default(); n_witnesses];
    assignments[0] = assignment;

    // solve witness
    let witness = compile_result
        .witness_solver
        .solve_witnesses(&assignments)
        .unwrap();

    // run/verify the circuit
    let output = compile_result.layered_circuit.run(&witness);
    assert!(output[0]);

    // create "circuit.txt"
    let file = std::fs::File::create("build/circuit.txt").unwrap();
    let writer = std::io::BufWriter::new(file);
    compile_result
        .layered_circuit
        .serialize_into(writer)
        .unwrap();

    // create "witness.txt"
    let file = std::fs::File::create("build/witness.txt").unwrap();
    let writer = std::io::BufWriter::new(file);
    witness.serialize_into(writer).unwrap();
}
