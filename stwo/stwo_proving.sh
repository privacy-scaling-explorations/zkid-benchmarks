#!/bin/bash

set -e

# Configuration
STWO_REPO="https://github.com/starkware-libs/stwo-cairo.git"
STWO_DIR="stwo-cairo"
PROVER_DIR="$STWO_DIR/stwo_cairo_prover"

SHA256_BENCHMARK_FILE="src/main.cairo"
# INPUT_FILE="src/input0.json" # "hello world" string
INPUT_FILE="src/input1.json" # random string of 2048 bytes(2KB)

BUILD_DIR="build"
COMPILED_FILE="build/main_compiled.json"
TRACE_FILE="build/trace.bin"
MEMORY_FILE="build/memory.bin"
PUB_INPUT="build/air_public_inputs.json"
PRIV_INPUT="build/air_private_inputs.json"
PROOF_FILE="build/proof.json"

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

# Step 0: Activate virtual environment
source .venv/bin/activate

# Step 1: Compile the Cairo program
echo "Compiling $SHA256_BENCHMARK_FILE..."
COMPILE_MEM=$(measure_memory cairo-compile $SHA256_BENCHMARK_FILE --output $COMPILED_FILE --proof_mode)
echo "Compilation memory usage: $(bytes_to_human $COMPILE_MEM)"

# Step 2: Run the program and generate execution trace
echo "Running the Cairo program..."
RUN_MEM=$(measure_memory cairo-run --program=$COMPILED_FILE \
          --program_input=$INPUT_FILE \
          --layout=starknet \
          --trace_file=$TRACE_FILE \
          --memory_file=$MEMORY_FILE \
          --air_public_input=$PUB_INPUT \
          --air_private_input=$PRIV_INPUT \
          --proof_mode)
echo "Execution memory usage: $(bytes_to_human $RUN_MEM)"

# Step 3: Clone stwo-cairo repository if not present
if [ ! -d "$STWO_DIR" ]; then
  echo "Cloning stwo-cairo repository..."
  git clone $STWO_REPO
fi

# Step 4: Generate STARK proof using Stwo
echo "Generating STARK proof with Stwo..."

ADAPTED_STWO_BIN="$PROVER_DIR/target/release/adapted_stwo"
if [ ! -f "$ADAPTED_STWO_BIN" ]; then
  echo "Building adapted_stwo..."
  cd $PROVER_DIR
  cargo build --release
  cd -
else
  echo "adapted_stwo binary already exists. Skipping build."
fi

START_TIME=$(date +%s)
/usr/bin/time -l "$ADAPTED_STWO_BIN" \
  --pub_json $PUB_INPUT \
  --priv_json $PRIV_INPUT \
  --proof_path $PROOF_FILE 2> prove_metrics.txt
END_TIME=$(date +%s)

# Calculate proof generation time
PROVE_TIME=$((END_TIME - START_TIME))
echo "Proof generation time: $PROVE_TIME seconds"

# Extract memory usage during proof generation
PROVE_MEM=$(awk '/maximum resident set size/ {print $1}' prove_metrics.txt)
echo "Proof generation memory usage: $(bytes_to_human $PROVE_MEM)"

# Step 5: Output performance metrics
echo "=== Performance Metrics ==="

# Proof size
PROOF_SIZE=$(stat -f%z $PROOF_FILE)
echo "Proof size: $PROOF_SIZE bytes"

# Total size of data needed for proof generation
DATA_SIZE=$(du -ck $TRACE_FILE $MEMORY_FILE $PUB_INPUT $PRIV_INPUT | grep total | awk '{print $1}')
echo "Total data size for proof generation: $DATA_SIZE KB"

# Extract Peak Memory Footprint (in bytes)
PEAK_MEM_BYTES=$(awk '/peak memory footprint/ {print $1}' prove_metrics.txt)

# Convert and display the memory metrics
echo "Peak Memory Footprint: $(bytes_to_human $PEAK_MEM_BYTES)"

# Summary
echo "--- Summary ---"
echo "Compilation(cairo-compile) memory usage: $(bytes_to_human $COMPILE_MEM)"
echo "Execution(cairo-run) memory usage: $(bytes_to_human $RUN_MEM)"
echo "Total data size for proof generation: $DATA_SIZE KB"

echo "STWO proof generation time: $PROVE_TIME seconds"
echo "STWO proof generation memory usage: $(bytes_to_human $PROVE_MEM)"
echo "STWO proof size: $(bytes_to_human $PROOF_SIZE)"

# Clean up metric files if desired
# rm prove_metrics.txt