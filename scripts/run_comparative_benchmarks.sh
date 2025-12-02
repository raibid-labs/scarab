#!/usr/bin/env bash
set -e

# Configuration
BENCH_CMD="./benchmarks/scarab-bench/target/release/scarab-bench"
RESULTS_FILE="BENCHMARK_RESULTS.md"
TERMINALS=("scarab-client" "alacritty" "wezterm" "kitty" "foot" "ghostty")

# Ensure benchmark tool is built
echo "Building benchmark tool..."
cd benchmarks/scarab-bench
cargo build --release
cd ../..

# Initialize Results File
echo "# Terminal Benchmark Results" > $RESULTS_FILE
echo "Date: $(date)" >> $RESULTS_FILE
echo "" >> $RESULTS_FILE
echo "| Terminal | Flood (MB/s) | Cursor (ops/s) | TUI (FPS) | Startup (ms) |" >> $RESULTS_FILE
echo "|----------|--------------|----------------|-----------|--------------|" >> $RESULTS_FILE

echo "Starting benchmarks..."

for term in "${TERMINALS[@]}"; do
    if ! command -v $term &> /dev/null; then
        echo "⚠️  Skipping $term (not found)"
        continue
    fi

    echo "Testing $term..."
    
    # 1. Flood Benchmark
    # We run this and parse stderr for "Speed: X MB/s"
    # Note: Running inside the terminal is tricky from a script without manual intervention.
    # For automation, we usually pipe commands or use 'start -- command'.
    # Since specific flags vary (e.g. -e, --), we need a helper function.
    
    run_in_term() {
        local t=$1
        local cmd=$2
        local out_file=$3
        
        case $t in
            "scarab-client")
                # Scarab currently doesn't support -e (Command execution) fully yet?
                # Assuming it runs whatever is in config or shell.
                # Actually, Scarab is in alpha. We might need to manually run this.
                # For now, we assume standard "-e" or similar if implemented.
                # If not implemented, we can't automate this yet!
                # Fallback: Run locally if term is "local" (for baseline)
                if [ "$t" == "local" ]; then
                    $cmd 2> $out_file
                else
                    # TODO: Implement command passing for Scarab
                    echo "0.00" > $out_file
                fi
                ;;
            "alacritty")
                alacritty -e sh -c "$cmd 2> $out_file"
                ;;
            "wezterm")
                wezterm start -- sh -c "$cmd 2> $out_file"
                ;;
            "kitty")
                kitty sh -c "$cmd 2> $out_file"
                ;;
            *)
                echo "Unknown runner for $t"
                ;;
        esac
    }

    # NOTE: Since fully automating GUI terminal execution and closing is complex and flaky
    # (waiting for window to appear, close after exit, etc.), for this first pass,
    # we will run the benchmarks LOCALLY (stdout/stderr) to establish a baseline for "Terminal logic speed"
    # vs "Renderer speed".
    
    # Wait, the user wants comparative stats. 
    # If I can't launch them, I can't measure them.
    # I will provide the manual instruction mode.
    
    FLOOD_RES="N/A"
    CURSOR_RES="N/A"
    TUI_RES="N/A"
    STARTUP_RES="N/A"

    # Measure Startup (rudimentary)
    # Time how long it takes to run 'true' or '--version'
    start_ts=$(date +%s%N)
    $term --version &> /dev/null || true
    end_ts=$(date +%s%N)
    # dur_ms=$(( (end_ts - start_ts) / 1000000 ))
    # STARTUP_RES="$dur_ms"
    
    # Since we can't automate cleanly without user setup, we'll log this limitation
    echo "| $term | Manual Run Req | Manual Run Req | Manual Run Req | Manual Run Req |" >> $RESULTS_FILE

done

echo "" >> $RESULTS_FILE
echo "## Instructions for Comparative Testing" >> $RESULTS_FILE
echo "To get the numbers for the table above, run the following commands inside each terminal:" >> $RESULTS_FILE
echo "" >> $RESULTS_FILE
echo "1. **Flood (Throughput):**" >> $RESULTS_FILE
echo "   \
$BENCH_CMD flood --size-mb 500\
" >> $RESULTS_FILE
echo "2. **Cursor (Latency/Update):**" >> $RESULTS_FILE
echo "   \
$BENCH_CMD cursor --count 50000\
" >> $RESULTS_FILE
echo "3. **TUI (FPS):**" >> $RESULTS_FILE
echo "   \
$BENCH_CMD tui --duration 10\
" >> $RESULTS_FILE
echo "" >> $RESULTS_FILE

echo "Done. See $RESULTS_FILE"
chmod +x $BENCH_CMD
