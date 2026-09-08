// `zkevm_hashes` re-exports `halo2_proofs` privately, so halo2 types come from `halo2_base`.
use halo2_base::{
    halo2_proofs::{
        SerdeFormat,
        halo2curves::{
            bn256::{Bn256, Fr, G1Affine},
            ff::PrimeField,
        },
        plonk::{Circuit, Error, ProvingKey, create_proof, keygen_pk, keygen_vk, verify_proof},
        poly::{
            commitment::ParamsProver,
            kzg::{
                commitment::{KZGCommitmentScheme, ParamsKZG},
                multiopen::{ProverSHPLONK, VerifierSHPLONK},
                strategy::SingleStrategy,
            },
        },
        transcript::{
            Blake2bRead, Blake2bWrite, Challenge255, TranscriptReadBuffer, TranscriptWriterBuffer,
        },
    },
    utils::fs::gen_srs,
};
use rand_core::OsRng;
use zkevm_hashes::{
    keccak::vanilla::{
        KeccakConfigParams,
        keccak_packed_multi::{get_keccak_capacity, get_num_keccak_f},
        param::{NUM_ROUNDS as KECCAK_NUM_ROUNDS, NUM_WORDS_TO_ABSORB as KECCAK_WORDS_TO_ABSORB},
    },
    sha256::vanilla::{
        param::SHA256_NUM_ROWS,
        util::{get_num_sha2_blocks, get_sha2_capacity},
    },
};

use crate::circuits::{KeccakCircuit, Sha256BitCircuit};

const MAX_K: u32 = 25;

/// Rows per keccak_f round. Trades circuit width against height; 28 is the value
/// upstream benchmarks use for the larger of their two test configurations.
const KECCAK_ROWS_PER_ROUND: usize = 28;

/// A circuit and its key material, ready for proving.
pub struct Prepared<C: Circuit<Fr> + Clone> {
    params: ParamsKZG<Bn256>,
    pk: ProvingKey<G1Affine>,
    circuit: C,
    public_digest: [Fr; 2],
    /// Rows required by the requested hash, excluding padding up to `2^k`.
    used_rows: usize,
}

fn domain_fits(k: u32, required_rows: usize, minimum_rows: usize, unusable_rows: usize) -> bool {
    let domain_rows = 1usize << k;
    domain_rows >= minimum_rows && required_rows <= domain_rows.saturating_sub(unusable_rows)
}

fn smallest_k(fits: impl Fn(u32) -> bool) -> u32 {
    (0..=MAX_K)
        .find(|&k| fits(k))
        .unwrap_or_else(|| panic!("input does not fit in a circuit of degree <= {MAX_K}"))
}

fn sha256_fits(k: u32, required_rows: usize) -> bool {
    if (1usize << k) < required_rows {
        return false;
    }
    let (minimum_rows, unusable_rows) = Sha256BitCircuit::minimum_rows_and_unusable_rows();
    domain_fits(k, required_rows, minimum_rows, unusable_rows)
}

fn keccak_config(k: u32) -> KeccakConfigParams {
    KeccakConfigParams {
        k,
        rows_per_round: KECCAK_ROWS_PER_ROUND,
    }
}

fn keccak_fits(k: u32, required_rows: usize) -> bool {
    if (1usize << k) < required_rows {
        return false;
    }
    let (minimum_rows, unusable_rows) =
        KeccakCircuit::minimum_rows_and_unusable_rows(keccak_config(k));
    domain_fits(k, required_rows, minimum_rows, unusable_rows)
}

/// Degree and used-row count for a SHA-256 circuit over `input_size` bytes.
pub fn sha256_dimensions(input_size: usize) -> (u32, usize) {
    let blocks = get_num_sha2_blocks(input_size);
    let rows = blocks * SHA256_NUM_ROWS;
    debug_assert_eq!(get_sha2_capacity(rows), blocks);
    (smallest_k(|k| sha256_fits(k, rows)), rows)
}

/// Degree and used-row count for a Keccak circuit over `input_size` bytes.
pub fn keccak_dimensions(input_size: usize) -> (u32, usize) {
    let permutations = get_num_keccak_f(input_size);
    // Inverse of `get_keccak_capacity`: a dummy round, the absorb lookahead window,
    // and `NUM_ROUNDS + 1` rounds per permutation, all scaled by rows per round.
    let rows = (1 + KECCAK_WORDS_TO_ABSORB + permutations * (KECCAK_NUM_ROUNDS + 1))
        * KECCAK_ROWS_PER_ROUND;
    debug_assert_eq!(
        get_keccak_capacity(rows, KECCAK_ROWS_PER_ROUND),
        permutations
    );
    let k = smallest_k(|k| keccak_fits(k, rows));
    (k, rows)
}

fn digest_instances(digest: &[u8]) -> [Fr; 2] {
    let digest: &[u8; 32] = digest.try_into().expect("hash digest must be 32 bytes");
    let hi = u128::from_be_bytes(digest[..16].try_into().unwrap());
    let lo = u128::from_be_bytes(digest[16..].try_into().unwrap());
    [Fr::from_u128(lo), Fr::from_u128(hi)]
}

fn keygen<C: Circuit<Fr> + Clone>(
    k: u32,
    circuit: C,
    public_digest: [Fr; 2],
    used_rows: usize,
) -> Prepared<C> {
    let params = gen_srs(k);
    keygen_with_params(params, circuit, public_digest, used_rows)
}

fn keygen_with_params<C: Circuit<Fr> + Clone>(
    params: ParamsKZG<Bn256>,
    circuit: C,
    public_digest: [Fr; 2],
    used_rows: usize,
) -> Prepared<C> {
    let shape = circuit.without_witnesses();
    let vk = keygen_vk(&params, &shape).expect("vk generation failed");
    let pk = keygen_pk(&params, vk, &shape).expect("pk generation failed");
    Prepared {
        params,
        pk,
        circuit,
        public_digest,
        used_rows,
    }
}

pub fn sha256_prepare(input_size: usize) -> Prepared<Sha256BitCircuit> {
    let (msg, digest) = utils::generate_sha256_input(input_size);
    let (k, used_rows) = sha256_dimensions(input_size);
    keygen(
        k,
        Sha256BitCircuit::new(used_rows, vec![msg]),
        digest_instances(&digest),
        used_rows,
    )
}

pub fn keccak_prepare(input_size: usize) -> Prepared<KeccakCircuit> {
    let (msg, digest) = utils::generate_keccak_input(input_size);
    let (k, used_rows) = keccak_dimensions(input_size);
    let config = keccak_config(k);
    keygen(
        k,
        KeccakCircuit::new(config, used_rows, vec![msg]),
        digest_instances(&digest),
        used_rows,
    )
}

pub fn prove<C: Circuit<Fr> + Clone>(prepared: &Prepared<C>) -> Vec<u8> {
    let mut transcript = Blake2bWrite::<_, _, Challenge255<_>>::init(vec![]);
    let instance_columns: [&[Fr]; 1] = [&prepared.public_digest];
    let instances: [&[&[Fr]]; 1] = [&instance_columns];
    create_proof::<
        KZGCommitmentScheme<Bn256>,
        ProverSHPLONK<'_, Bn256>,
        Challenge255<_>,
        _,
        Blake2bWrite<Vec<u8>, G1Affine, _>,
        _,
    >(
        &prepared.params,
        &prepared.pk,
        std::slice::from_ref(&prepared.circuit),
        &instances,
        OsRng,
        &mut transcript,
    )
    .expect("proving failed");
    transcript.finalize()
}

// The harness types these closures as `FnMut(&PreparedContext, &Proof)` and
// `FnMut(&Proof) -> usize` with `Proof = Vec<u8>`, so the parameter must be
// `&Vec<u8>` exactly; `&[u8]` does not satisfy the bound.
#[allow(clippy::ptr_arg)]
pub fn verify<C: Circuit<Fr> + Clone>(prepared: &Prepared<C>, proof: &Vec<u8>) {
    verify_result(prepared, proof).expect("verification failed");
}

fn verify_result<C: Circuit<Fr> + Clone>(
    prepared: &Prepared<C>,
    proof: &[u8],
) -> Result<(), Error> {
    let mut transcript = Blake2bRead::<_, _, Challenge255<_>>::init(proof);
    let instance_columns: [&[Fr]; 1] = [&prepared.public_digest];
    let instances: [&[&[Fr]]; 1] = [&instance_columns];
    verify_proof::<
        KZGCommitmentScheme<Bn256>,
        VerifierSHPLONK<'_, Bn256>,
        Challenge255<G1Affine>,
        Blake2bRead<&[u8], G1Affine, Challenge255<G1Affine>>,
        SingleStrategy<'_, Bn256>,
    >(
        prepared.params.verifier_params(),
        prepared.pk.get_vk(),
        SingleStrategy::new(&prepared.params),
        &instances,
        &mut transcript,
    )
}

/// Row budget required by the requested hash, excluding Halo2's unusable rows and
/// padding up to `2^k`.
pub fn num_constraints<C: Circuit<Fr> + Clone>(prepared: &Prepared<C>) -> usize {
    prepared.used_rows
}

/// Serialized proving key size. The KZG SRS is universal rather than circuit-specific,
/// so it is not counted here.
pub fn preprocessing_size<C: Circuit<Fr> + Clone>(prepared: &Prepared<C>) -> usize {
    prepared.pk.to_bytes(SerdeFormat::RawBytes).len()
}

#[allow(clippy::ptr_arg)]
pub fn proof_size(proof: &Vec<u8>) -> usize {
    proof.len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use halo2_base::halo2_proofs::dev::MockProver;

    #[test]
    fn dimensions_fit_and_are_minimal() {
        let expected = [
            (128, (8, 216), (11, 1_204)),
            (256, (9, 360), (11, 1_904)),
            (512, (10, 648), (12, 3_304)),
            (1_024, (11, 1_224), (13, 6_104)),
            (2_048, (12, 2_376), (14, 11_704)),
        ];

        for (size, expected_sha256, expected_keccak) in expected {
            let (k, rows) = sha256_dimensions(size);
            assert_eq!((k, rows), expected_sha256);
            assert_eq!(
                get_sha2_capacity(rows),
                get_num_sha2_blocks(size),
                "sha256 {size}: capacity does not match the requested message"
            );
            assert!(sha256_fits(k, rows), "sha256 {size}: k={k} too small");
            assert!(
                k == 0 || !sha256_fits(k - 1, rows),
                "sha256 {size}: k={k} larger than needed"
            );

            let (k, rows) = keccak_dimensions(size);
            assert_eq!((k, rows), expected_keccak);
            let permutations = get_num_keccak_f(size);
            assert_eq!(
                get_keccak_capacity(rows, KECCAK_ROWS_PER_ROUND),
                permutations,
                "keccak {size}: capacity does not match the requested message"
            );
            assert!(keccak_fits(k, rows), "keccak {size}: k={k} too small");
            assert!(
                k == 0 || !keccak_fits(k - 1, rows),
                "keccak {size}: k={k} larger than needed"
            );
        }
    }

    #[test]
    fn public_digest_is_bound_to_private_message() {
        let size = 128;

        let (sha_message, sha_digest) = utils::generate_sha256_input(size);
        let (sha_k, sha_rows) = sha256_dimensions(size);
        let sha_instances = digest_instances(&sha_digest).to_vec();
        let sha_circuit = Sha256BitCircuit::new(sha_rows, vec![sha_message.clone()]);
        MockProver::run(sha_k, &sha_circuit, vec![sha_instances.clone()])
            .unwrap()
            .assert_satisfied();

        let mut wrong_sha_digest = sha_instances.clone();
        wrong_sha_digest[0] += Fr::from(1);
        assert!(
            MockProver::run(sha_k, &sha_circuit, vec![wrong_sha_digest])
                .unwrap()
                .verify()
                .is_err()
        );

        let mut wrong_sha_message = sha_message;
        wrong_sha_message[0] ^= 1;
        let wrong_sha_circuit = Sha256BitCircuit::new(sha_rows, vec![wrong_sha_message]);
        assert!(
            MockProver::run(sha_k, &wrong_sha_circuit, vec![sha_instances])
                .unwrap()
                .verify()
                .is_err()
        );

        let (keccak_message, keccak_digest) = utils::generate_keccak_input(size);
        let (keccak_k, keccak_rows) = keccak_dimensions(size);
        let keccak_instances = digest_instances(&keccak_digest).to_vec();
        let keccak_circuit = KeccakCircuit::new(
            keccak_config(keccak_k),
            keccak_rows,
            vec![keccak_message.clone()],
        );
        MockProver::run(keccak_k, &keccak_circuit, vec![keccak_instances.clone()])
            .unwrap()
            .assert_satisfied();

        let mut wrong_keccak_digest = keccak_instances.clone();
        wrong_keccak_digest[0] += Fr::from(1);
        assert!(
            MockProver::run(keccak_k, &keccak_circuit, vec![wrong_keccak_digest])
                .unwrap()
                .verify()
                .is_err()
        );

        let mut wrong_keccak_message = keccak_message;
        wrong_keccak_message[0] ^= 1;
        let wrong_keccak_circuit = KeccakCircuit::new(
            keccak_config(keccak_k),
            keccak_rows,
            vec![wrong_keccak_message],
        );
        assert!(
            MockProver::run(keccak_k, &wrong_keccak_circuit, vec![keccak_instances])
                .unwrap()
                .verify()
                .is_err()
        );
    }

    #[test]
    fn kzg_verifier_uses_public_digest() {
        let size = 128;
        let (message, digest) = utils::generate_sha256_input(size);
        let (k, rows) = sha256_dimensions(size);
        let circuit = Sha256BitCircuit::new(rows, vec![message]);
        let mut prepared = keygen_with_params(
            ParamsKZG::<Bn256>::setup(k, OsRng),
            circuit,
            digest_instances(&digest),
            rows,
        );
        let proof = prove(&prepared);

        assert!(verify_result(&prepared, &proof).is_ok());
        prepared.public_digest[0] += Fr::from(1);
        assert!(verify_result(&prepared, &proof).is_err());
    }
}
