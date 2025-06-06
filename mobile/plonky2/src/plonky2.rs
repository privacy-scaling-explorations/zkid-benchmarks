use std::{error::Error, str::FromStr};

use mopro_ffi::GenerateProofResult;
use num_bigint::BigUint;
use plonky2::{
    field::{
        extension::Extendable,
        goldilocks_field::GoldilocksField,
        types::{Field, PrimeField},
    },
    hash::hash_types::RichField,
    plonk::{
        circuit_data::{ProverCircuitData, ProverOnlyCircuitData, VerifierCircuitData},
        proof::ProofWithPublicInputs,
    },
    util::serialization::{Buffer, GateSerializer, Read, WitnessGeneratorSerializer, Write},
};
use plonky2::{
    iop::witness::PartialWitness,
    plonk::config::{GenericConfig, PoseidonGoldilocksConfig},
};

pub fn plonky2_prove<GTSer, GnSer, const D: usize, C, F: RichField + Extendable<D>>(
    prover_data_path: &str,
    gate_serializer: &GTSer,
    generator_serializer: &GnSer,
    generate_witness: impl Fn(&ProverOnlyCircuitData<F, C, D>) -> PartialWitness<F>,
) -> Result<GenerateProofResult, Box<dyn Error>>
where
    C: GenericConfig<D, F = F>,
    GTSer: GateSerializer<F, D>,
    GnSer: WitnessGeneratorSerializer<F, D>,
{
    let pk_bytes = std::fs::read(prover_data_path)?;
    let prover_data: ProverCircuitData<F, C, D> =
        ProverCircuitData::from_bytes(&pk_bytes, gate_serializer, generator_serializer).unwrap();

    let pw = generate_witness(&prover_data.prover_only);

    let proof_with_public_inputs = prover_data.prove(pw)?;

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

pub fn serialize_inputs(public_inputs: &[String]) -> Vec<u8> {
    const D: usize = 2;
    type C = PoseidonGoldilocksConfig;
    type F = <C as GenericConfig<D>>::F;
    let mut public_inputs_buffer = Vec::new();
    public_inputs_buffer
        .write_usize(public_inputs.len())
        .unwrap();
    public_inputs_buffer
        .write_field_vec(
            &public_inputs
                .iter()
                .map(|x| F::from_noncanonical_biguint(BigUint::from_str(x).unwrap()))
                .collect::<Vec<_>>(),
        )
        .unwrap();
    public_inputs_buffer
}

pub fn deserialize_inputs(buffer: &[u8]) -> Vec<String> {
    let mut buffer = Buffer::new(buffer);
    let len = buffer.read_usize().unwrap();
    let field_vec: Vec<GoldilocksField> = buffer.read_field_vec(len).unwrap();
    field_vec
        .iter()
        .map(|x| x.to_canonical_biguint().to_string())
        .collect()
}

pub fn plonky2_verify<GTSer, const D: usize, C, F: RichField + Extendable<D>>(
    verifier_data_path: &str,
    serialized_proof: Vec<u8>,
    serialized_inputs: Vec<u8>,
    gate_serializer: &GTSer,
) -> Result<bool, Box<dyn Error>>
where
    C: GenericConfig<D, F = F>,
    GTSer: GateSerializer<F, D>,
{
    let vk_bytes = std::fs::read(verifier_data_path)?;

    let verifier_data: VerifierCircuitData<F, C, D> =
        VerifierCircuitData::from_bytes(vk_bytes, gate_serializer).unwrap();

    let proof = ProofWithPublicInputs::from_bytes(
        [serialized_proof, serialized_inputs].concat(),
        &verifier_data.common,
    )
    .unwrap();

    let verify = verifier_data.verify(proof);

    Ok(verify.is_ok())
}
