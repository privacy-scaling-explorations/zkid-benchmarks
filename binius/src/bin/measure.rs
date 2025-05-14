use anyhow::Error;
use binius::bench::{prove, sha256_no_lookup_prepare, verify};

fn main() -> Result<(), Error> {
    let allocator = bumpalo::Bump::new();
    let (constraint_system, args, witness, backend) = sha256_no_lookup_prepare(&allocator);

    let (cs, args, proof) = prove(constraint_system, args, witness, backend);

    verify(cs, args, proof);

    Ok(())
}
