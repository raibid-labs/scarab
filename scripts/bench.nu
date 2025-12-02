#!/usr/bin/env nu

# Benchmark Configuration
const BENCH_DIR = "benchmarks/scarab-bench"
const BENCH_BIN = $("benchmarks/scarab-bench")/target/release/scarab-bench
const RESULTS_FILE = "BENCHMARK_RESULTS.md"

# Terminal configurations: Name mapping to launch command structure
# Note: command is a list of strings to prepend to the actual benchmark command
const TERMINALS = [
    { name: "alacritty", args: ["-e"] },
    { name: "wezterm", args: ["start", "--"] },
    { name: "kitty", args: [] },
    { name: "foot", args: [] },
    { name: "ghostty", args: ["-e"] },
    # Scarab command integration is WIP, attempting standard -e convention or direct PTY pass-through
    { name: "scarab-client", args: ["--command"] }
]

def build_bench_tool [] {
    print "🔨 Building benchmark tool..."
    cd $BENCH_DIR
    cargo build --release
    cd ../..
}

# Generates the shell command string to run the benchmark and redirect output
def generate_wrapped_cmd [bench_type: string, params: string, out_file: string] {
    let abs_bench_bin = ($BENCH_BIN | path expand)
    let abs_out_file = ($out_file | path expand)
    
    # We wrap in sh -c to ensure redirection happens *inside* the terminal
    # and not in the launching shell.
    # We also force a small sleep to ensure the terminal has time to initialize rendering contexts
    $"sh -c '($abs_bench_bin) ($bench_type) ($params) > ($abs_out_file) 2>&1'"
}

def parse_result [file_path: string, metric_key: string] {
    if not ($file_path | path exists) {
        return "Err:NoOutput"
    }
    
    let content = (open $file_path)
    # Parse lines looking for "  Speed: 123.45 MB/s" or "  FPS: 60.00"
    let lines = ($content | lines)
    
    let match = ($lines | where { |it| $it =~ $metric_key } | first)
    
    if ($match | is-empty) {
        return "Err:Parse"
    }
    
    # Extract number (naive but works for our format "  Key: Value Unit")
    # Split by ':' then space
    let parts = ($match | split row ":")
    if ($parts | length) < 2 { return "Err:Fmt" }
    
    ($parts | get 1 | str trim | split row " " | get 0)
}

def run_test [term_config: record, bench_type: string, params: string, metric_key: string] {
    let term_cmd = $term_config.name
    
    # Check if terminal exists
    if (which $term_cmd | is-empty) {
        return "N/A (Missing)"
    }

    let tmp_file = (mktemp --suffix .txt)
    let wrapped_cmd = (generate_wrapped_cmd $bench_type $params $tmp_file)
    
    print $("   ...running ($bench_type) on ($term_cmd)")

    # Construct the full execution command list
    let full_cmd = ($term_config.args | append $wrapped_cmd)
    
    # Launch terminal
    # We run it in background or wait? 
    # Most terminals detach. We need to wait for the output file to be populated.
    try {
        run-external $term_cmd ...$full_cmd
    } catch {
        return "Err:Launch"
    }

    # Wait loop (max 30s)
    let mut attempts = 0
    let max_attempts = 60 # 30s * 0.5s
    
    loop {
        if ($tmp_file | path exists) {
            let size = (ls $tmp_file | get size | get 0)
            # Check if file has content and the process might be done
            # Ideally the benchmark tool writes everything at the end.
            if $size > 0b {
                # Give it a split second to flush
                sleep 500ms
                break
            }
        }
        
        sleep 500ms
        $attempts = $attempts + 1
        if $attempts > $max_attempts {
            return "Err:Timeout"
        }
    }

    let result = (parse_result $tmp_file $metric_key)
    rm $tmp_file
    return $result
}

def main [] {
    build_bench_tool

    # Prepare Markdown Table
    let header = "| Terminal | Flood (MB/s) | Cursor (ops/s) | TUI (FPS) |"
    let separator = "|---|---|---|---|"
    
    mut results = []

    for term in $TERMINALS {
        print $("👉 Testing ($term.name)...")
        
        # 1. Flood Test (100MB)
        let flood_res = (run_test $term "flood" "--size-mb 100" "Speed")
        
        # 2. Cursor Test (10k ops)
        let cursor_res = (run_test $term "cursor" "--count 10000" "Speed")
        
        # 3. TUI Test (5 seconds)
        let tui_res = (run_test $term "tui" "--duration 5" "FPS")

        $results = ($results | append {
            term: $term.name,
            flood: $flood_res,
            cursor: $cursor_res,
            tui: $tui_res
        })
    }

    # Output Table
    print "\n📊 RESULTS\n"
    print $header
    print $separator
    
    for r in $results {
        print $("| ($r.term) | ($r.flood) | ($r.cursor) | ($r.tui) |")
    }
    
    # Save to file
    let table_rows = ($results | each { |r| $("| ($r.term) | ($r.flood) | ($r.cursor) | ($r.tui) |") } | str join "\n")
    let content = $("# Benchmark Results\nDate: (date now)\n\n($header)\n($separator)\n($table_rows)")
    $content | save -f $RESULTS_FILE
    
    print $"\n✅ Saved results to ($RESULTS_FILE)"
}
