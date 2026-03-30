#!/usr/bin/env bash
set -euo pipefail

# Run criterion benchmarks, append results to CSV history, and generate benchmarks.md
#
# Usage:
#   ./scripts/bench-history.sh              # defaults to bench-history.csv
#   ./scripts/bench-history.sh results.csv  # custom output file

HISTORY_FILE="${1:-bench-history.csv}"
BENCHMARKS_MD="benchmarks.md"
TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
COMMIT=$(git rev-parse --short HEAD 2>/dev/null || echo "unknown")
BRANCH=$(git branch --show-current 2>/dev/null || echo "unknown")

# Create header if file doesn't exist
if [ ! -f "$HISTORY_FILE" ]; then
    echo "timestamp,commit,branch,benchmark,estimate_ns" > "$HISTORY_FILE"
fi

echo "╔══════════════════════════════════════════╗"
echo "║       sharira benchmark suite            ║"
echo "╠══════════════════════════════════════════╣"
echo "║  commit: $COMMIT                          ║"
echo "║  branch: $BRANCH                            ║"
echo "║  date:   $TIMESTAMP   ║"
echo "╚══════════════════════════════════════════╝"
echo ""

# Run benchmarks and capture output, stripping ANSI escape codes
BENCH_OUTPUT=$(cargo bench --bench benchmarks -- --output-format bencher 2>/dev/null | sed 's/\x1b\[[0-9;]*m//g')

# Show full output
echo "$BENCH_OUTPUT"
echo ""

# Collect results for CSV and markdown
declare -a BENCH_NAMES=()
declare -a BENCH_NS=()

while IFS= read -r line; do
    if [[ "$line" =~ ^test\ (.+)\ \.\.\.\ bench:\ +([0-9]+)\ ns/iter ]]; then
        name="${BASH_REMATCH[1]}"
        ns="${BASH_REMATCH[2]}"
        BENCH_NAMES+=("$name")
        BENCH_NS+=("$ns")
        echo "$TIMESTAMP,$COMMIT,$BRANCH,$name,$ns" >> "$HISTORY_FILE"
    fi
done <<< "$BENCH_OUTPUT"

COUNT=${#BENCH_NAMES[@]}
echo "Appended $COUNT benchmarks to $HISTORY_FILE"

# Generate benchmarks.md with three-point tracking
{
    echo "# Benchmarks"
    echo ""
    echo "Generated: $TIMESTAMP | Commit: $COMMIT | Branch: $BRANCH"
    echo ""
    echo "| Benchmark | Latest (ns) | Previous (ns) | Baseline (ns) |"
    echo "|-----------|-------------|---------------|---------------|"

    for ((i = 0; i < COUNT; i++)); do
        name="${BENCH_NAMES[$i]}"
        latest="${BENCH_NS[$i]}"

        # Find previous and baseline from history
        previous=$(grep ",$name," "$HISTORY_FILE" | tail -2 | head -1 | cut -d',' -f5 || echo "$latest")
        baseline=$(grep ",$name," "$HISTORY_FILE" | head -1 | cut -d',' -f5 || echo "$latest")

        echo "| $name | $latest | $previous | $baseline |"
    done
} > "$BENCHMARKS_MD"

echo "Generated $BENCHMARKS_MD"
