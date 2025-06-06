//
//  ContentView.swift
//  mopro-test
//
//  Created by Chance on 6/25/24.
//
import SwiftUI
import moproFFI

func serializeOutputs(_ stringArray: [String]) -> [UInt8] {
    var bytesArray: [UInt8] = []
    let length = stringArray.count
    var littleEndianLength = length.littleEndian
    let targetLength = 32
    withUnsafeBytes(of: &littleEndianLength) {
        bytesArray.append(contentsOf: $0)
    }
    for value in stringArray {
        // TODO: should handle 254-bit input
        var littleEndian = Int32(value)!.littleEndian
        var byteLength = 0
        withUnsafeBytes(of: &littleEndian) {
            bytesArray.append(contentsOf: $0)
            byteLength = byteLength + $0.count
        }
        if byteLength < targetLength {
            let paddingCount = targetLength - byteLength
            let paddingArray = [UInt8](repeating: 0, count: paddingCount)
            bytesArray.append(contentsOf: paddingArray)
        }
    }
    return bytesArray
}


struct ContentView: View {
    @State private var textViewText = ""
    @State private var isPlonky2ProveButtonEnabled = true
    @State private var isPlonky2VerifyButtonEnabled = false
    @State private var generatedPlonky2Proof: GenerateProofResult?
    private let proverDataPath = Bundle.main.path(forResource: "plonky2_fibonacci_pk.bin", ofType: "")!
    private let verifierDataPath = Bundle.main.path(forResource: "plonky2_fibonacci_vk.bin", ofType: "")!
    
    var body: some View {
        VStack(spacing: 10) {
            Image(systemName: "globe")
                .imageScale(.large)
                .foregroundStyle(.tint)
            Button("Bench Plonky2 Sha256", action: runPlonky2ProveAction).disabled(!isPlonky2ProveButtonEnabled).accessibilityIdentifier("provePlonky2")
            //Button("Verify Plonky2", action: runPlonky2VerifyAction).disabled(!isPlonky2VerifyButtonEnabled).accessibilityIdentifier("verifyPlonky2")

            ScrollView {
                Text(textViewText)
                    .padding()
                    .accessibilityIdentifier("proof_log")
            }
            .frame(height: 200)
        }
        .padding()
    }
}

extension ContentView {
    
    func runPlonky2ProveAction() {
        textViewText += "Generating Plonky2 proof... "
        do {
            // Prepare inputs
            var inputs = [String: [String]]()
            let a = 0
            let b = 1
            inputs["a"] = [String(a)]
            inputs["b"] = [String(b)]
            
            //let start = CFAbsoluteTimeGetCurrent()
            
            // Generate Proof
            let generateProofResult = try sha256RoundtripBench()
            //assert(!generateProofResult.proof.isEmpty, "Proof should not be empty")
            
            //let end = CFAbsoluteTimeGetCurrent()
            //let timeTaken = end - start
            
            // Store the generated proof and public inputs for later verification
            //generatedPlonky2Proof = generateProofResult
            
            textViewText += generateProofResult[0]//"\(String(format: "%.3f", timeTaken))s 1️⃣\n"
            textViewText += "\n"
            textViewText += generateProofResult[1]
            textViewText += "\n"
            textViewText += generateProofResult[2]
            
            isPlonky2VerifyButtonEnabled = true
        } catch {
            textViewText += "\nProof generation failed: \(error.localizedDescription)\n"
        }
    }
    
    func runPlonky2VerifyAction() {
        guard let proof = generatedPlonky2Proof else {
            textViewText += "Proof has not been generated yet.\n"
            return
        }
        
        textViewText += "Verifying Plonky2 proof... "
        do {
            let start = CFAbsoluteTimeGetCurrent()
            
            let isValid = try verifySha256Proof(
                verifierDataPath: verifierDataPath, proof: proof.proof, inputs: proof.inputs)
            let end = CFAbsoluteTimeGetCurrent()
            let timeTaken = end - start
            
            if isValid {
                textViewText += "\(String(format: "%.3f", timeTaken))s 2️⃣\n"
            } else {
                textViewText += "\nProof verification failed.\n"
            }
            isPlonky2VerifyButtonEnabled = false
        } catch let error as MoproError {
            print("\nMoproError: \(error)")
        } catch {
            print("\nUnexpected error: \(error)")
        }
    }
}

