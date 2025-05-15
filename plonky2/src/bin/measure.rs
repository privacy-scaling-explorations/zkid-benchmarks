#[cfg(not(target_env = "msvc"))]
use tikv_jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

use memory_stats::memory_stats;
use plonky2::{plonk::config::PoseidonGoldilocksConfig, util::serialization::Write};
use plonky2_sha256::bench::{prove, sha256_no_lookup_prepare};
use plonky2_u32::gates::arithmetic_u32::{U32GateSerializer, U32GeneratorSerializer};
use tikv_jemalloc_ctl::raw;
use tikv_jemalloc_ctl::{
    epoch,
    stats::{self},
};

const D: usize = 2;
type C = PoseidonGoldilocksConfig;

fn main() {
    epoch::advance().unwrap();
    let allocated_before = stats::allocated::read().unwrap();
    let resident_before = stats::resident::read().unwrap();
    let usage_before = memory_stats().unwrap();

    let (data, pw) = sha256_no_lookup_prepare();

    let usage_after = memory_stats().unwrap();
    println!(
        "memory_stats: Preprocessing: {} GB resident | {} GB virt",
        (usage_after.physical_mem - usage_before.physical_mem) as f32 / (1024.0 * 1024.0 * 1024.0),
        (usage_after.virtual_mem - usage_before.virtual_mem) as f32 / (1024.0 * 1024.0 * 1024.0)
    );
    epoch::advance().unwrap();
    let allocated_after = stats::allocated::read().unwrap();
    let resident_after = stats::resident::read().unwrap();
    println!(
        "jemalloc: Preprocessing: {} GB alloc | {} GB resident",
        (allocated_after - allocated_before) as f32 / 1024.0 / 1024.0 / 1024.0,
        (resident_after - resident_before) as f32 / 1024.0 / 1024.0 / 1024.0,
    );

    let gate_serializer = U32GateSerializer;
    let common_data_size = data.common.to_bytes(&gate_serializer).unwrap().len();
    let generator_serializer = U32GeneratorSerializer::<C, D>::default();
    let prover_data_size = data
        .prover_only
        .to_bytes(&generator_serializer, &data.common)
        .unwrap()
        .len();

    println!(
        "Common data size: {}B, Prover data size: {}B",
        common_data_size, prover_data_size
    );
    let _ = unsafe { raw::write(b"stats.print\0", &true) };
    epoch::advance().unwrap();
    let allocated_before = stats::allocated::read().unwrap();
    let resident_before = stats::resident::read().unwrap();

    let usage_before = memory_stats().unwrap();

    let proof = prove(&data.prover_data(), pw);

    let usage_after = memory_stats().unwrap();
    println!(
        "memory_stats: Proving: {} GB resident | {} GB virt",
        (usage_after.physical_mem - usage_before.physical_mem) as f32 / (1024.0 * 1024.0 * 1024.0),
        (usage_after.virtual_mem - usage_before.virtual_mem) as f32 / (1024.0 * 1024.0 * 1024.0)
    );

    epoch::advance().unwrap();
    let allocated_after = stats::allocated::read().unwrap();
    let resident_after = stats::resident::read().unwrap();
    println!(
        "Proving: {} GB alloc | {} GB resident",
        (allocated_after - allocated_before) as f32 / 1024.0 / 1024.0 / 1024.0,
        (resident_after - resident_before) as f32 / 1024.0 / 1024.0 / 1024.0,
    );

    let mut buffer = Vec::new();
    buffer.write_proof(&proof.proof).unwrap();
    println!("Proof size: {} KB", buffer.len() as f32 / 1024.0);
}
