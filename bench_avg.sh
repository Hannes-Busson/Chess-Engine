#!/bin/bash
cargo build --release 2>/dev/null
RUNS=${1:-7}
BINARY=./target/release/chess_engine
total_nps=0
total_nodes=0
for i in $(seq 1 $RUNS); do
    line=$(echo "benchmt" | $BINARY 2>/dev/null | grep "^combined bench:")
    nodes=$(echo "$line" | awk '{print $3}')
    nps=$(echo "$line" | awk '{print $5}')
    echo "run $i: $nodes nodes, $nps nps"
    total_nps=$((total_nps + nps))
    total_nodes=$((total_nodes + nodes))
done
echo "---"
echo "average: $((total_nodes / RUNS)) nodes, $((total_nps / RUNS)) nps over $RUNS runs"
