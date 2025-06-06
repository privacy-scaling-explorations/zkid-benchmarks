use plonky2::iop::witness::{PartialWitness, WitnessWrite};
use plonky2::plonk::{circuit_builder::CircuitBuilder, circuit_data::CircuitConfig};
use plonky2_sha256::circuit::{array_to_bits, make_circuits};
use plonky2_u32::gates::arithmetic_u32::{U32GateSerializer, U32GeneratorSerializer};
use std::io::Error;

use plonky2::plonk::config::{GenericConfig, PoseidonGoldilocksConfig};
use plonky2::{field::goldilocks_field::GoldilocksField, plonk::circuit_data::CircuitData};

const EXPECTED_RES: [u8; 256] = [
    0, 0, 1, 1, 1, 1, 0, 0, 0, 1, 0, 1, 1, 1, 1, 0, 1, 0, 1, 0, 0, 0, 1, 1, 1, 1, 1, 0, 1, 0, 0, 1,
    0, 0, 1, 0, 1, 0, 0, 0, 1, 1, 1, 1, 1, 0, 0, 1, 0, 0, 1, 1, 0, 1, 1, 0, 0, 1, 0, 0, 0, 0, 1, 1,
    1, 1, 1, 0, 0, 1, 1, 0, 1, 1, 1, 1, 0, 1, 0, 1, 0, 1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 1, 0, 1, 1, 0,
    1, 0, 0, 1, 1, 0, 0, 1, 0, 0, 0, 1, 0, 1, 1, 0, 1, 0, 1, 1, 1, 0, 1, 0, 1, 0, 0, 1, 1, 1, 1, 1,
    0, 1, 1, 1, 0, 0, 0, 1, 0, 1, 0, 1, 0, 1, 1, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 1, 1, 1, 0, 0, 0, 0,
    1, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0, 0, 1, 1, 1, 0, 0, 0, 0, 1, 0, 1, 0, 1, 0, 1,
    1, 0, 1, 1, 0, 1, 0, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 1, 0, 1, 1, 1, 0, 1, 0, 1, 1, 1,
    0, 1, 0, 1, 1, 1, 0, 1, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0,
];

fn main() -> Result<(), Error> {
    let mut msg = vec![0; 128_usize];
    for (i, msg_byte) in msg.iter_mut().enumerate().take(127) {
        *msg_byte = i as u8;
    }

    let msg_bits = array_to_bits(&msg);
    let len = msg.len() * 8;
    const D: usize = 2;
    type C = PoseidonGoldilocksConfig;
    type F = <C as GenericConfig<D>>::F;
    let mut builder = CircuitBuilder::<F, D>::new(CircuitConfig::standard_recursion_zk_config());
    let targets = make_circuits(&mut builder, len as u64);
    let mut pw: PartialWitness<F> = PartialWitness::new();

    for (i, msg_bit) in msg_bits.iter().enumerate().take(len) {
        pw.set_bool_target(targets.message[i], *msg_bit).unwrap();
    }

    for (i, expected_bit) in EXPECTED_RES.iter().enumerate() {
        if *expected_bit == 1 {
            builder.assert_one(targets.digest[i].target);
        } else {
            builder.assert_zero(targets.digest[i].target);
        }
    }

    let circuit_data = builder.build::<C>();
    let gate_serializer = U32GateSerializer;
    let generator_serializer = U32GeneratorSerializer::<C, D>::default();

    let circuit_bytes = circuit_data.to_bytes(&gate_serializer, &generator_serializer);

    let circuit_bytes = circuit_bytes.unwrap();

    let prover_data = circuit_data.prover_data();
    let pk_bytes = prover_data
        .to_bytes(&gate_serializer, &generator_serializer)
        .unwrap();

    let circuit_data: CircuitData<GoldilocksField, C, 2> =
        CircuitData::from_bytes(&circuit_bytes, &gate_serializer, &generator_serializer).unwrap();
    let verifier_data = circuit_data.verifier_data();
    let vk_bytes = verifier_data.to_bytes(&gate_serializer).unwrap();

    std::fs::write("zk-artifacts/plonky2_sha256_pk.bin", pk_bytes)?;
    std::fs::write("zk-artifacts/plonky2_sha256_vk.bin", vk_bytes)?;

    Ok(())
}
