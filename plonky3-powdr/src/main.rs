use powdr::Session;

fn main() {
    env_logger::init();

    //let MSG_BYTE_SIZE = 2048;
    // let mut msg = vec![0u8; MSG_BYTE_SIZE];
    // let mut rng = rand::rng();
    // for msg_bit in msg.iter_mut().take(MSG_BYTE_SIZE - 1) {
    //     *msg_bit = rng.random_range(0..=1);
    // }
    // // hash the message
    // let mut hasher = Sha256::new();
    // hasher.update(&msg);
    // let result = hasher.finalize();

    // Create a new powdr session to make proofs for the `guest` crate.
    // Store all temporary and final artifacts in `powdr-target`.
    let mut session = Session::builder()
        .guest_path("./guest")
        .out_path("powdr-target")
        // powdrVM splits long execution traces into chunks
        // which are proven individually.
        // The default size of a chunk is 2^20 = 1048576 rows.
        // For experiments and smaller traces/proofs, it may be beneficial to reduce the chunk size.
        // Create a new powdr session with a custom chunk size.
        // 2^18 = 262144 rows per chunk.
        .chunk_size_log2(18)
        .build()
        // Write `some_data` to channel 1 and the sum of `some_data` to channel 2.
        // Any serde-serializable type can be written to a channel.
        //.write(1, &msg)
        ;

    // Fast dry run to test execution.
    //session.run();

    // Uncomment to compute the proof.
    session.prove();
}
