#!/bin/bash
set -eo pipefail

# Add CPU optimization flags
export RUSTFLAGS="-Ctarget-cpu=native"

# Build release with native optimizations
echo "Building with RUSTFLAGS: $RUSTFLAGS"
cargo build --release --example prove_prime_field_31

# Generate timestamp-based log name
TIMESTAMP=$(date +%s)
LOG_FILE="benchmark-$TIMESTAMP.log"

# Run benchmark with tee to both console and log
target/release/examples/prove_prime_field_31 \
  --field baby-bear \
  --objective poseidon2-permutations \
  --log-trace-length 16 \
  --discrete-fourier-transform radix-2-dit-parallel \
  --merkle-hash poseidon-2 2>&1 | tee "$LOG_FILE"

# Error-checking extraction
GEN_TIME=$(grep "generate vectorized" "$LOG_FILE" | grep -oP '\[\s*\K[0-9.]+(?=(ms|s))' | head -1)
PROVE_TIME=$(grep "prove \[ " "$LOG_FILE" | grep -oP '\[\s*\K[0-9.]+(?=s)')
VERIFY_TIME=$(grep "verify \[ " "$LOG_FILE" | grep -oP '\[\s*\K[0-9.]+(?=(ms|s))')

if [[ -z "$GEN_TIME" ]]; then
  echo "❌ Error: Failed to extract trace generation time"
  exit 1
fi

if [[ -z "$PROVE_TIME" ]]; then
  echo "❌ Error: Failed to extract proving time"
  exit 1
fi

if [[ -z "$VERIFY_TIME" ]]; then
  echo "❌ Error: Failed to extract verification time"
  exit 1
fi

echo "Benchmark Results:"
echo "Trace Generation: ${GEN_TIME}ms"
echo "Proving Time: ${PROVE_TIME}s"
echo "Verification Time: ${VERIFY_TIME}ms"

# After metric extraction
JSON_DATA=$(jq -n \
  --arg sha "$GITHUB_SHA" \
  --arg date "$(date -u +"%Y-%m-%dT%H:%M:%SZ")" \
  --arg branch "$GITHUB_REF_NAME" \
  --arg workflow "$GITHUB_WORKFLOW" \
  --arg gen "$GEN_TIME" \
  --arg prove "$PROVE_TIME" \
  --arg verify "$VERIFY_TIME" \
  '{
    commit: $sha,
    timestamp: $date,
    branch: $branch,
    workflow: $workflow,
    metrics: {
      trace_gen_ms: ($gen | tonumber),
      prove_time_s: ($prove | tonumber),
      verify_time_ms: ($verify | tonumber)
    },
    system: {
      runner: "${{ runner.os }}",
      rustc: "${{ steps.rustc-version.outputs.version }}",
      cpu: "${{ steps.cpu-info.outputs.model }}"
    }
  }')

echo "$JSON_DATA" > metrics.json
