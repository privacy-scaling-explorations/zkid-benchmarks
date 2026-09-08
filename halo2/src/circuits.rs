// `zkevm_hashes` re-exports `halo2_proofs` privately (`use halo2_base::halo2_proofs;`),
// so halo2 types have to be imported from `halo2_base` instead.
use halo2_base::halo2_proofs::{
    circuit::{Layouter, SimpleFloorPlanner},
    halo2curves::bn256::Fr,
    plonk::{Circuit, Column, ConstraintSystem, Error, Expression, Fixed, Instance},
    poly::Rotation,
};
use zkevm_hashes::{
    keccak::vanilla::{
        KeccakCircuitConfig, KeccakConfigParams, keccak_packed_multi::get_keccak_capacity,
        witness::multi_keccak,
    },
    sha256::vanilla::{
        columns::Sha256CircuitConfig,
        param::{RATE as SHA256_RATE, SHA256_NUM_ROWS},
        util::get_sha2_capacity,
    },
};

/// SHA-256 over one private message with a public digest.
#[derive(Clone, Default)]
pub struct Sha256BitCircuit {
    inputs: Vec<Vec<u8>>,
    input_len: usize,
    hash_rows: usize,
}

impl Sha256BitCircuit {
    pub fn new(hash_rows: usize, inputs: Vec<Vec<u8>>) -> Self {
        assert_eq!(
            inputs.len(),
            1,
            "sha256 circuit requires one private message"
        );
        let input_len = inputs[0].len();
        Self {
            inputs,
            input_len,
            hash_rows,
        }
    }

    pub(crate) fn minimum_rows_and_unusable_rows() -> (usize, usize) {
        let mut meta = ConstraintSystem::<Fr>::default();
        let _ = <Self as Circuit<Fr>>::configure(&mut meta);
        (meta.minimum_rows(), meta.blinding_factors() + 1)
    }
}

impl Circuit<Fr> for Sha256BitCircuit {
    type Config = (Sha256CircuitConfig<Fr>, Column<Instance>, Column<Fixed>);
    type FloorPlanner = SimpleFloorPlanner;
    type Params = ();

    fn without_witnesses(&self) -> Self {
        Self {
            inputs: vec![],
            input_len: self.input_len,
            hash_rows: self.hash_rows,
        }
    }

    fn configure(meta: &mut ConstraintSystem<Fr>) -> Self::Config {
        let hash = Sha256CircuitConfig::new(meta);
        let instance = meta.instance_column();
        meta.enable_equality(instance);
        let expected_length = meta.fixed_column();
        let length = hash.hash_table.length;
        meta.create_gate("sha256 message length", |meta| {
            let encoded_length = meta.query_fixed(expected_length, Rotation::cur());
            let length = meta.query_advice(length, Rotation::cur());
            vec![
                encoded_length.clone()
                    * (length + Expression::Constant(Fr::from(1)) - encoded_length),
            ]
        });
        (hash, instance, expected_length)
    }

    fn synthesize(
        &self,
        (hash, instance, expected_length): Self::Config,
        mut layouter: impl Layouter<Fr>,
    ) -> Result<(), Error> {
        let [digest_lo, digest_hi] = layouter.assign_region(
            || "sha256 circuit",
            |mut region| {
                let assigned_blocks = hash.multi_sha256(
                    &mut region,
                    self.inputs.clone(),
                    Some(get_sha2_capacity(self.hash_rows)),
                );
                for (block_index, _) in assigned_blocks.iter().enumerate() {
                    // `is_final` is not equality-enabled upstream, so pinning each
                    // cumulative length forces the selected last block to be final.
                    let cumulative_len = self.input_len.min((block_index + 1) * SHA256_RATE);
                    let encoded_len = u64::try_from(cumulative_len + 1)
                        .expect("sha256 input length must fit in u64");
                    region.assign_fixed(
                        expected_length,
                        (block_index + 1) * SHA256_NUM_ROWS - 1,
                        Fr::from(encoded_len),
                    );
                }
                let digest = assigned_blocks.last().ok_or(Error::Synthesis)?.output();
                Ok([digest.lo().cell(), digest.hi().cell()])
            },
        )?;
        layouter.constrain_instance(digest_lo, instance, 0);
        layouter.constrain_instance(digest_hi, instance, 1);
        Ok(())
    }
}

/// Keccak-256 over one private message with a public digest.
#[derive(Clone, Default)]
pub struct KeccakCircuit {
    config: KeccakConfigParams,
    inputs: Vec<Vec<u8>>,
    input_len: usize,
    hash_rows: usize,
}

impl KeccakCircuit {
    pub fn new(config: KeccakConfigParams, hash_rows: usize, inputs: Vec<Vec<u8>>) -> Self {
        assert_eq!(
            inputs.len(),
            1,
            "keccak circuit requires one private message"
        );
        let input_len = inputs[0].len();
        Self {
            config,
            inputs,
            input_len,
            hash_rows,
        }
    }

    pub(crate) fn minimum_rows_and_unusable_rows(config: KeccakConfigParams) -> (usize, usize) {
        let mut meta = ConstraintSystem::<Fr>::default();
        let _ = <Self as Circuit<Fr>>::configure_with_params(&mut meta, config);
        (meta.minimum_rows(), meta.blinding_factors() + 1)
    }
}

impl Circuit<Fr> for KeccakCircuit {
    type Config = (KeccakCircuitConfig<Fr>, Column<Instance>);
    type FloorPlanner = SimpleFloorPlanner;
    type Params = KeccakConfigParams;

    fn params(&self) -> Self::Params {
        self.config
    }

    fn without_witnesses(&self) -> Self {
        Self {
            config: self.config,
            inputs: vec![],
            input_len: self.input_len,
            hash_rows: self.hash_rows,
        }
    }

    fn configure_with_params(
        meta: &mut ConstraintSystem<Fr>,
        params: Self::Params,
    ) -> Self::Config {
        // The keccak config only allocates SecondPhase advice columns; halo2 requires
        // at least one FirstPhase column to exist, so add an empty one.
        meta.advice_column();
        let hash = KeccakCircuitConfig::new(meta, params);
        let instance = meta.instance_column();
        meta.enable_equality(instance);
        let constant = meta.fixed_column();
        meta.enable_constant(constant);
        (hash, instance)
    }

    fn configure(_: &mut ConstraintSystem<Fr>) -> Self::Config {
        unreachable!("keccak circuit is configured via `configure_with_params`")
    }

    fn synthesize(
        &self,
        (hash, instance): Self::Config,
        mut layouter: impl Layouter<Fr>,
    ) -> Result<(), Error> {
        let params = hash.parameters;
        hash.load_aux_tables(&mut layouter, params.k)?;
        let [digest_lo, digest_hi] = layouter.assign_region(
            || "keccak circuit",
            |mut region| {
                let (witness, _) = multi_keccak(
                    &self.inputs,
                    Some(get_keccak_capacity(self.hash_rows, params.rows_per_round)),
                    params,
                );
                let assigned_rows = hash.assign(&mut region, &witness);
                // `multi_keccak` prepends one dummy round.
                let input_len_cell = assigned_rows
                    .get(params.rows_per_round)
                    .ok_or(Error::Synthesis)?
                    .bytes_left
                    .cell();
                let output_offset = assigned_rows
                    .len()
                    .checked_sub(params.rows_per_round)
                    .ok_or(Error::Synthesis)?;
                let digest = assigned_rows.get(output_offset).ok_or(Error::Synthesis)?;
                let input_len = u64::try_from(self.input_len).map_err(|_| Error::Synthesis)?;
                region.constrain_constant(input_len_cell, Fr::from(input_len))?;
                region.constrain_constant(digest.is_final.cell(), Fr::from(1))?;
                Ok([digest.hash_lo.cell(), digest.hash_hi.cell()])
            },
        )?;
        layouter.constrain_instance(digest_lo, instance, 0);
        layouter.constrain_instance(digest_hi, instance, 1);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use halo2_base::halo2_proofs::{dev::MockProver, halo2curves::ff::PrimeField};

    #[test]
    fn keccak_input_length_is_bound() {
        let input_size = 128;
        let (message, digest) = utils::generate_keccak_input(input_size);
        let (k, hash_rows) = crate::bench::keccak_dimensions(input_size);
        let config = KeccakConfigParams {
            k,
            rows_per_round: 28,
        };
        let digest: &[u8; 32] = digest.as_slice().try_into().unwrap();
        let instances = vec![
            Fr::from_u128(u128::from_be_bytes(digest[16..].try_into().unwrap())),
            Fr::from_u128(u128::from_be_bytes(digest[..16].try_into().unwrap())),
        ];
        let mut circuit = KeccakCircuit::new(config, hash_rows, vec![message]);

        MockProver::run(k, &circuit, vec![instances.clone()])
            .unwrap()
            .assert_satisfied();
        circuit.input_len -= 1;
        assert!(
            MockProver::run(k, &circuit, vec![instances])
                .unwrap()
                .verify()
                .is_err()
        );
    }
}
