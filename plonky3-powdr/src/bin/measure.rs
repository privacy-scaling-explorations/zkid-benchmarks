use sha::bench::{prepare_pipeline, prove, verify};
#[cfg(not(target_env = "msvc"))]
use tikv_jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

use memory_stats::memory_stats;
use tikv_jemalloc_ctl::{epoch, stats};

fn main() {
    epoch::advance().unwrap();
    let allocated_before = stats::allocated::read().unwrap();
    let resident_before = stats::resident::read().unwrap();
    let usage_before = memory_stats().unwrap();

    let mut pipeline = prepare_pipeline();

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

    epoch::advance().unwrap();
    let allocated_before = stats::allocated::read().unwrap();
    let resident_before = stats::resident::read().unwrap();

    let usage_before = memory_stats().unwrap();

    prove(&mut pipeline);

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

    verify(pipeline);
}
