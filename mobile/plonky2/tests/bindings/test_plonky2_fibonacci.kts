import uniffi.mopro.*

try {
    val vkPath = "./zk-artifacts/plonky2_fibonacci_vk.bin"
    val pkPath = "./zk-artifacts/plonky2_fibonacci_pk.bin"

    // Prepare inputs
    val inputs = mutableMapOf<String, List<String>>()
    inputs["a"] = listOf("0")
    inputs["b"] = listOf("1")

    // Generate proof
    var generateProofResult = generateFibonacciProof(pkPath, inputs)
    assert(generateProofResult.proof.size > 0) { "Proof is empty" }
    assert(generateProofResult.inputs.size > 0) { "Inputs are empty" }

    // Verify proof
    var isValid = verifyFibonacciProof(vkPath, generateProofResult.proof, generateProofResult.inputs)
    assert(isValid) { "Proof is invalid" }


} catch (e: Exception) {
    println(e)
    throw e
}
