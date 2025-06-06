import Foundation
import mopro

do {
  let timesStrings = try sha256RoundtripBench()
  print("SHA256 roundtrip bench: \(timesStrings)")
} catch let error as MoproError {
  print("MoproError: \(error)")
  throw error
} catch {
  print("Unexpected error: \(error)")
  throw error
}
