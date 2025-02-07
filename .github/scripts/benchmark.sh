#!/bin/bash

# Build release version
cargo build --release --example prove_prime_field_31

# Run benchmark and capture output
target/release/examples/prove_prime_field_31 \
  --field baby-bear \
  --objective poseidon2-permutations \
  --log-trace-length 16 \
  --discrete-fourier-transform radix-2-dit-parallel \
  --merkle-hash poseidon-2 2>&1 | tee benchmark.log

# Extract metrics
GEN_TIME=$(grep "generate vectorized" benchmark.log | awk '{print $(NF-1)}' | sed 's/ms//')
PROVE_TIME=$(grep "prove \[ " benchmark.log | awk '{print $(NF-1)}' | sed 's/s//')
VERIFY_TIME=$(grep "verify \[ " benchmark.log | awk '{print $(NF-1)}' | sed 's/s//')

echo "Benchmark Results:"
echo "Trace Generation: ${GEN_TIME}ms"
echo "Proving Time: ${PROVE_TIME}s"
echo "Verification Time: ${VERIFY_TIME}s"
