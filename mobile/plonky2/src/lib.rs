uniffi::include_scaffolding!("mopro");

use ::plonky2::field::types::Field;
use ::plonky2::plonk::circuit_builder::CircuitBuilder;
use ::plonky2::plonk::circuit_data::{CircuitConfig, ProverOnlyCircuitData};
use ::plonky2::util::serialization::Write;
use ::plonky2::{
    iop::witness::{PartialWitness, WitnessWrite},
    plonk::config::{GenericConfig, PoseidonGoldilocksConfig},
    util::serialization::{DefaultGateSerializer, DefaultGeneratorSerializer},
};
use mopro_ffi::GenerateProofResult;
use num_bigint::BigUint;
use plonky2_sha256::circuit::{array_to_bits, make_circuits};
use plonky2_u32::gates::arithmetic_u32::U32GateSerializer;
use std::str::FromStr;
use thiserror::Error;

mod plonky2;

#[derive(Debug, Error)]
pub enum MoproError {
    #[error("Plonky2Error: {0}")]
    Plonky2Error(String),
}

fn generate_fibonacci_proof(
    prover_data_path: String,
    inputs: std::collections::HashMap<String, Vec<String>>,
) -> Result<GenerateProofResult, MoproError> {
    const D: usize = 2;
    type C = PoseidonGoldilocksConfig;
    type F = <C as GenericConfig<D>>::F;
    let gate_serializer = DefaultGateSerializer;
    let generator_serializer = DefaultGeneratorSerializer::<C, D>::default();

    let generate_witness = |prover_only_circuit_data: &ProverOnlyCircuitData<F, C, D>| {
        let mut pw = PartialWitness::new();
        let a = F::from_noncanonical_biguint(BigUint::from_str(&inputs["a"][0]).unwrap());
        let b = F::from_noncanonical_biguint(BigUint::from_str(&inputs["b"][0]).unwrap());

        pw.set_target(prover_only_circuit_data.public_inputs[0], a)
            .unwrap();
        pw.set_target(prover_only_circuit_data.public_inputs[1], b)
            .unwrap();
        pw
    };

    plonky2::plonky2_prove::<DefaultGateSerializer, DefaultGeneratorSerializer<C, D>, D, C, F>(
        &prover_data_path,
        &gate_serializer,
        &generator_serializer,
        &generate_witness,
    )
    .map_err(|e| MoproError::Plonky2Error(format!("plonky2 error: {}", e)))
}

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

fn generate_sha256_proof() -> Result<GenerateProofResult, MoproError> {
    let mut msg = vec![0; 2048_usize];
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
    let mut pw = PartialWitness::new();

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

    let data = builder.build::<C>();

    let proof_with_public_inputs = data.prove(pw).unwrap();

    let mut proof_buffer = Vec::new();
    proof_buffer
        .write_proof(&proof_with_public_inputs.proof)
        .unwrap();
    let mut public_inputs_buffer = Vec::new();
    public_inputs_buffer
        .write_usize(proof_with_public_inputs.public_inputs.len())
        .unwrap();
    public_inputs_buffer
        .write_field_vec(&proof_with_public_inputs.public_inputs)
        .unwrap();

    Ok(GenerateProofResult {
        proof: proof_buffer,
        inputs: public_inputs_buffer,
    })
}

fn sha256_roundtrip_bench() -> Result<Vec<String>, MoproError> {
    let start = std::time::Instant::now();
    let mut msg = vec![0; 2048_usize];
    for (i, msg_byte) in msg.iter_mut().enumerate().take(127) {
        *msg_byte = i as u8;
    }

    let msg_bits = array_to_bits(&msg);
    let len = msg.len() * 8;
    const D: usize = 2;
    type C = PoseidonGoldilocksConfig;
    type F = <C as GenericConfig<D>>::F;
    let mut builder = CircuitBuilder::<F, D>::new(CircuitConfig::standard_recursion_zk_config());
    println!("msg_len_in_bits: {}", len as u64);
    let targets = make_circuits(&mut builder, len as u64);
    let mut pw = PartialWitness::new();

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

    let end = std::time::Instant::now();
    let circuit_definition_phase = (end - start).as_millis();
    println!("circuit_definition_phase: {:?}", circuit_definition_phase);
    let start = std::time::Instant::now();
    let data = builder.build_prover::<C>();
    let end = std::time::Instant::now();
    let circuit_build_phase = (end - start).as_millis();
    println!("circuit_build_phase: {:?}", circuit_build_phase);
    let start = std::time::Instant::now();
    let _proof_with_public_inputs = data.prove(pw).unwrap();
    let end = std::time::Instant::now();
    let witgen_and_proving_phase = (end - start).as_millis();
    println!("witgen_and_proving_phase: {:?}", witgen_and_proving_phase);
    // let data = builder.build_verifier();
    // // Proof verification phase
    // let start = std::time::Instant::now();
    // data.verify(proof_with_public_inputs).unwrap();
    // // End of the proof verification phase
    // let end = std::time::Instant::now();
    // let elapsed4 = (end - start).as_millis();
    // println!("Proof verification phase: {:?}", end - start);

    let res = vec![
        format!("Circuit definition: {:?} ms ", circuit_definition_phase),
        format!("Circuit build: {:?} s", circuit_build_phase),
        format!("Witgen + proving: {:?} s", witgen_and_proving_phase),
        // format!("Proof verification phase: {:?}", elapsed4),
    ];
    Ok(res)
}

fn verify_fibonacci_proof(
    verifier_data_path: String,
    proof: Vec<u8>,
    inputs: Vec<u8>,
) -> Result<bool, MoproError> {
    const D: usize = 2;
    type C = PoseidonGoldilocksConfig;
    type F = <C as GenericConfig<D>>::F;
    let gate_serializer = DefaultGateSerializer;
    plonky2::plonky2_verify::<DefaultGateSerializer, D, C, F>(
        &verifier_data_path,
        proof,
        inputs,
        &gate_serializer,
    )
    .map_err(|e| MoproError::Plonky2Error(format!("error verifying proof: {}", e)))
}

fn verify_sha256_proof(
    verifier_data_path: String,
    proof: Vec<u8>,
    inputs: Vec<u8>,
) -> Result<bool, MoproError> {
    const D: usize = 2;
    type C = PoseidonGoldilocksConfig;
    type F = <C as GenericConfig<D>>::F;
    let gate_serializer = U32GateSerializer;
    plonky2::plonky2_verify::<U32GateSerializer, D, C, F>(
        &verifier_data_path,
        proof,
        inputs,
        &gate_serializer,
    )
    .map_err(|e| MoproError::Plonky2Error(format!("error verifying proof: {}", e)))
}

fn deserialize_inputs(buffer: Vec<u8>) -> Vec<String> {
    plonky2::deserialize_inputs(&buffer)
}

fn serialize_inputs(public_inputs: Vec<String>) -> Vec<u8> {
    plonky2::serialize_inputs(&public_inputs)
}
