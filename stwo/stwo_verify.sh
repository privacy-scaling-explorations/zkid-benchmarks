#!/bin/bash

set -e

# Configuration
STWO_REPO="https://github.com/starkware-libs/stwo-cairo.git"
STWO_DIR="stwo-cairo"
PROVER_DIR="$STWO_DIR/stwo_cairo_prover"

BUILD_DIR="build"
PUB_INPUT="build/air_public_inputs.json"
PRIV_INPUT="build/air_private_inputs.json"
PROOF_FILE="build/proof.json"

# Check if build directory exists
if [ ! -d "$BUILD_DIR" ]; then
  echo "Error: build directory not found."
  exit 1
fi

# Check if stwo-cairo directory exists
if [ ! -d "$STWO_DIR" ]; then
  echo "Error: stwo-cairo directory not found."
  exit 1
fi

# Verify STARK proof using Stwo
echo "Verifying STARK proof with Stwo..."

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
  --proof_path $PROOF_FILE \
  --verify 2> verify_metrics.txt
END_TIME=$(date +%s)

# Calculate proof verification time
VERIFY_TIME=$((END_TIME - START_TIME))
echo "Proof verification time: $VERIFY_TIME seconds"

# Summary
echo "--- Summary ---"

echo "STWO proof verification time: $VERIFY_TIME seconds"

# Clean up metric files if desired
# rm verify_metrics.txt
