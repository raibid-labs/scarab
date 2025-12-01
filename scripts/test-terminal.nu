#!/usr/bin/env nu
# Test script for Scarab terminal - starts daemon and client

# Kill any existing processes
ps | where name =~ scarab | each { |proc| kill -9 $proc.pid }
sleep 1sec

# Clean up shared memory (if exists)
^ipcrm -M 0 | complete | ignore

print "=== Starting Daemon ==="
let daemon_log = "/tmp/daemon.log"
let daemon = (cargo run --release -p scarab-daemon
    o> $daemon_log e> $daemon_log &)

print $"Daemon started with PID: ($daemon)"

# Wait for daemon to initialize
sleep 5sec

# Show daemon status
print "\n=== Daemon Log (first 50 lines) ==="
open $daemon_log | lines | first 50 | str join "\n" | print

# Check shared memory
print "\n=== Shared Memory Status ==="
^ipcs -m | lines | where $it =~ scarab | str join "\n" | print

# Check socket
print "\n=== IPC Socket Status ==="
ls /tmp/scarab-daemon.sock | print

# Now start client
print "\n=== Starting Client (in foreground) ==="
cargo run --release -p scarab-client | tee /tmp/client.log
