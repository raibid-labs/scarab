// Phage Plugin for Scarab Terminal
// Status bar + AI context management
//
// Install: fpm add phage
// Repository: https://github.com/raibid-labs/scarab

// Log that Phage plugin is loading
Scarab.log "info" "Phage plugin loading..."

// Add Phage status indicator to left side of status bar
// Priority 100 means it appears early in the bar
Scarab.status_add "left" " Phage" 100

// Show a welcome notification
Scarab.notify "Phage Active" "AI context injection ready" "success"

// Log completion
Scarab.log "info" "Phage plugin loaded successfully"

// Return success
true
