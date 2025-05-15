#!/bin/bash

set -e

# Configuration
EXPANDER_REPO="https://github.com/PolyhedraZK/Expander.git"
EXPANDER_DIR="Expander"

BUILD_DIR="build"
CIRCUIT_FILE="build/circuit.txt"
WITNESS_FILE="build/witness.txt"
PROOF_FILE="build/proof.bin"


# Function to convert bytes to human-readable format
bytes_to_human() {
  local bytes=$1
  local kib=$((1024))
  local mib=$((1024 * kib))
  local gib=$((1024 * mib))

  if (( bytes >= gib )); then
    printf "%.2f GiB" "$(echo "$bytes / $gib" | bc -l)"
  elif (( bytes >= mib )); then
    printf "%.2f MiB" "$(echo "$bytes / $mib" | bc -l)"
  elif (( bytes >= kib )); then
    printf "%.2f KiB" "$(echo "$bytes / $kib" | bc -l)"
  else
    printf "%d B" "$bytes"
  fi
}

# Function to measure memory usage of a command
measure_memory() {
  /usr/bin/time -l "$@" 2>&1 | awk '/maximum resident set size/ {print $1}'
}


# Create "build" directory
if [ ! -d "$BUILD_DIR" ]; then
  echo "Creating build directory..."
  mkdir build
fi

# Step 1: Compile the circuit & get artifacts
echo "Step 1: Compiling the circuit..."
COMPILE_MEM=$(measure_memory cargo r --release)
echo "Compilation memory usage: $(bytes_to_human $COMPILE_MEM)"

# Step 2: Clone the Expander repository if it doesn't exist
if [ ! -d "$EXPANDER_DIR" ]; then
  echo "Step 2: Cloning the Expander repository..."
  git clone $EXPANDER_REPO
  cd $EXPANDER_DIR
  cargo run --bin=dev-setup --release
  cd -
fi

# Step 3: Run the Expander prover
echo "Step 3: Running the Expander prover..."

EXPANDER_EXEC_BIN="$EXPANDER_DIR/target/release/expander-exec"
if [ ! -f "$EXPANDER_EXEC_BIN" ]; then
  echo "Building expander-exec ..."
  cd $EXPANDER_DIR
  RUSTFLAGS="-C target-cpu=native" cargo build --release
  cd -
else
  echo "expander-exec binary already exists. Skipping build."
fi

PROVE_START_TIME=$(date +%s)
/usr/bin/time -l "$EXPANDER_EXEC_BIN" \
    -p Orion prove \
    -c $CIRCUIT_FILE \
    -w $WITNESS_FILE \
    -o $PROOF_FILE 2> prove_metrics.txt
PROVE_END_TIME=$(date +%s)

# Calculate proof generation time
PROVE_TIME=$((PROVE_END_TIME - PROVE_START_TIME))
echo "Proof generation time: $PROVE_TIME seconds"

# Extract memory usage during proof generation
PROVE_MEM=$(awk '/maximum resident set size/ {print $1}' prove_metrics.txt)
echo "Proof generation memory usage: $(bytes_to_human $PROVE_MEM)"

# Step 4: Run the Expander verifier
echo "Step 4: Running the Expander verifier..."
VERIFY_START_TIME=$(date +%s%N)
/usr/bin/time -l "$EXPANDER_EXEC_BIN" \
    -p Orion verify \
    -c $CIRCUIT_FILE \
    -w $WITNESS_FILE \
    -i $PROOF_FILE
VERIFY_END_TIME=$(date +%s%N)

# Calculate proof verification time
VERIFY_TIME_NS=$((VERIFY_END_TIME - VERIFY_START_TIME))
VERIFY_TIME_MS=$((VERIFY_TIME_NS / 1000000))
echo "Proof verification time: $VERIFY_TIME_MS milliseconds"


# Step 5: Output performance metrics
echo "=== Performance Metrics ==="

# Proof size
PROOF_SIZE=$(stat -f%z $PROOF_FILE)
echo "Proof size: $PROOF_SIZE bytes"

# Total size of data needed for proof generation
DATA_SIZE=$(du -ck $CIRCUIT_FILE $WITNESS_FILE | grep total | awk '{print $1}')
echo "Total data size for proof generation: $(bytes_to_human $DATA_SIZE)"

# Extract Peak Memory Footprint (in bytes)
PEAK_MEM_BYTES=$(awk '/peak memory footprint/ {print $1}' prove_metrics.txt)

# Convert and display the memory metrics
echo "Peak Memory Footprint: $(bytes_to_human $PEAK_MEM_BYTES)"

# Summary
echo "--- Summary ---"
echo "Circuit compilation memory usage: $(bytes_to_human $COMPILE_MEM)"
echo "Total data size for proof generation: $(bytes_to_human $DATA_SIZE)"

echo "Expander proof generation memory usage: $(bytes_to_human $PROVE_MEM)"
echo "Expander proof size: $(bytes_to_human $PROOF_SIZE)"

echo "Expander proof generation time: $PROVE_TIME seconds"
echo "Expander proof verification time: $VERIFY_TIME_MS milliseconds"
