// Here we're calling a macro exported with Uniffi. This macro will
// write some functions and bind them to FFI type. These
// functions will invoke the `get_circom_wtns_fn` generated below.
mopro_ffi::app!();

use std::time::Instant;

use examples::bench::{prove, sha256_no_lookup_prepare, verify};

#[derive(uniffi::Record)]
pub struct BiniusSha256Result {
    prepare_time: String,
    prove_time: String,
    verify_time: String,
}

// CIRCOM_TEMPLATE
#[uniffi::export]
pub fn binius_sha256() -> BiniusSha256Result {
    let allocator = bumpalo::Bump::new();
    let now = Instant::now();
    let (constraint_system, args, witness, backend) = sha256_no_lookup_prepare(&allocator);
    let prepare_time = now.elapsed();

    let now = Instant::now();
    let (cs, args, proof) = prove(constraint_system, args, witness, backend);
    let prove_time = now.elapsed();

    let now = Instant::now();
    verify(cs, args, proof);
    let verify_time = now.elapsed();

    println!("prepare time: {:?}", prepare_time);
    println!("prove time: {:?}", prove_time);
    println!("verify time: {:?}", verify_time);

    BiniusSha256Result {
        prepare_time: prepare_time.as_millis().to_string(),
        prove_time: prove_time.as_millis().to_string(),
        verify_time: verify_time.as_millis().to_string(),
    }
}
// HALO2_TEMPLATE

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binius_sha256() {
        binius_sha256();
    }
}
