// Phage Plugin for Scarab Terminal
// Status overlay showing Phage daemon connection status
//
// Install: fpm add phage
// Repository: https://github.com/raibid-labs/scarab

// Set window title to indicate Phage is active
Scarab.setWindowTitle "Scarab [Phage]"

// Add Phage status overlay in bottom-left corner
// Uses overlay API since status bar API requires full Fusabi runtime
Scarab.addOverlay "phage-status" BottomLeft (Text " Phage" 12.0 "#50fa7b")
