// These two lines are necessary for the program to properly compile.
//
// Under the hood, we wrap your main function with some extra code so that it behaves properly
// inside the zkVM.
#![no_main]
sp1_zkvm::entrypoint!(main);

use sha_lib::sha2;

pub fn main() {
    // Read an input to the program.
    //
    // Behind the scenes, this compiles down to a custom system call which handles reading inputs
    // from the prover.
    //let n = sp1_zkvm::io::read::<u32>();

    let input = &[5u8; 2048];
    let a = sha2(input);

    // Encode the public values of the program.
    //let bytes = PublicValuesStruct::abi_encode(&PublicValuesStruct { n, a });

    // Commit to the public values of the program. The final proof will have a commitment to all the
    // bytes that were committed to.
    //sp1_zkvm::io::commit_slice(&bytes);
}
