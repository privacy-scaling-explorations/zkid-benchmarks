//
//  ContentView.swift
//  MoproApp
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

struct HeaderTest: Decodable {
    let storage: [UInt8]
    let len: UInt32
}

struct PubKeyTest: Decodable {
    let modulus: [String]
    let redc: [String]
}

struct SequenceTest: Decodable {
    let index: UInt32
    let length: UInt32
}


struct ContentView: View {
    @State private var textViewText = ""
    @State private var isCircomProveButtonEnabled = true
    @State private var isCircomVerifyButtonEnabled = false
    @State private var isHalo2roveButtonEnabled = true
    @State private var isHalo2VerifyButtonEnabled = false
    @State private var isNoirProveButtonEnabled = true
    @State private var isNoirVerifyButtonEnabled = false
    @State private var generatedCircomProof: CircomProof?
    @State private var circomPublicInputs: [String]?
    @State private var generatedHalo2Proof: Data?
    @State private var halo2PublicInputs: Data?
    @State private var generatedNoirProof: Data?
    
    var body: some View {
        VStack(spacing: 10) {
            Image(systemName: "globe")
                .imageScale(.large)
                .foregroundStyle(.tint)
            Button("Prove Binius", action: runCircomProveAction).disabled(!isCircomProveButtonEnabled).accessibilityIdentifier("proveCircom")

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
    func runCircomProveAction() {
        textViewText += "Generating Binius proof... \n"
        do {
            // Prepare inputs
            let a = 3
            let b = 5
            let c = a*b
            let input_str: String = "{\"b\":[\"5\"],\"a\":[\"3\"]}"

            // Expected outputs
            let outputs: [String] = [String(c), String(a)]

            let start = CFAbsoluteTimeGetCurrent()

            // Generate Proof
            //let generateProofResult = try generateCircomProof(zkeyPath: zkeyPath, circuitInputs: input_str, proofLib: ProofLib.arkworks)
            let result = biniusSha256()
            //assert(!generateProofResult.proof.a.x.isEmpty, "Proof should not be empty")
            //assert(outputs == generateProofResult.inputs, "Circuit outputs mismatch the expected outputs")

            let end = CFAbsoluteTimeGetCurrent()
            let timeTaken = end - start

            textViewText += "Prepare time: \(result.prepareTime) ms 1️⃣\n"
            textViewText += "Proving time: \(result.proveTime) ms 1️⃣\n"
            textViewText += "Verification time: \(result.verifyTime) ms 1️⃣\n"

            isCircomVerifyButtonEnabled = true
        } catch {
            textViewText += "\nProof generation failed: \(error.localizedDescription)\n"
        }
    }
}

