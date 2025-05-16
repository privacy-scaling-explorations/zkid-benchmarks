# zkID Benchmarks

## Motivation

In many conventional digital identity systems, a credential issuer signs a SHA256 hash of some attributes, and the verifier should learn the preimage of the hash to check the signature. Many applications are exploring a zero-knowledge (ZK) based approach to digital identity that allows proving statements about the hash preimage without revealing the plaintext data. Many conventional digital identity systems use a combination of SHA256 hash function and ECDSA signature scheme. These cryptographic algorithms are known to be ZK-un-friendly and may introduce a bottleneck in the ZK-based digital identity systems. This repository contains a set of SHA256 benchmarks in various proving systems. The choice of proving systems was based on the following requirements:

| Requirement                                         | Rationale                                                  |
| --------------------------------------------------- | ---------------------------------------------------------- |
| No trusted setup / long structured reference string | Fewer security assumptions and less mobile bandwidth usage |
| Overall low bandwidth usage                         | Mobile device limitations                                  |
| Recursive proving                                   | Offline verifiable presentation generation capability      |
| Fast proving                                        | Mobile device limitations                                  |
| Post-quantum security                               | Future-proofing                                            |

## Benchmarks

The SHA256 benchmarks that use ZKVMs have minimal circuits that only hash the hardcoded input. We kept the circuits as simple as possible to minimize the ZKVM overhead, so the benchmarks better reflect the proving system performance on SHA256.

### Running the benchmarks

Please refer to the individual README files in each subfolder for instructions on how to run the benchmarks.
