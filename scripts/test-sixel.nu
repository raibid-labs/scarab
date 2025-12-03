#!/usr/bin/env nu
# Test script to generate Sixel graphics test sequences

def main [] {
    print "=== Sixel Graphics Test Sequences ==="
    print ""

    # Test 1: Simple red square
    print "Test 1: Simple red square (3x1 sixels)"
    print -n $"\e[P1;1q"                    # Start Sixel with 1:1 aspect ratio
    print -n "#1;2;100;0;0"                 # Define color 1 as red (RGB: 100,0,0)
    print -n "#1"                           # Select color 1
    print -n "~~~"                          # Draw 3 sixels (all 6 bits set)
    print -n $"\e\\"                        # End Sixel (ST)
    print ""
    print ""

    # Test 2: Blue and green stripes
    print "Test 2: Vertical stripes (blue, green, blue)"
    print -n $"\e[P1;1q"                    # Start Sixel
    print -n "#2;2;0;0;100"                 # Define color 2 as blue
    print -n "#3;2;0;100;0"                 # Define color 3 as green
    print -n "#2~~~#3~~~#2~~~"              # Draw alternating colors
    print -n $"\e\\"                        # End Sixel
    print ""
    print ""

    # Test 3: Multi-row pattern
    print "Test 3: Two-row pattern"
    print -n $"\e[P1;1q"                    # Start Sixel
    print -n "#1;2;100;0;0"                 # Red
    print -n "#4;2;100;100;0"               # Yellow
    print -n "#1~~~~"                       # First row: 4 red sixels
    print -n "-"                            # Line feed (next sixel row)
    print -n "#4~~~~"                       # Second row: 4 yellow sixels
    print -n $"\e\\"                        # End Sixel
    print ""
    print ""

    # Test 4: Repeat command test
    print "Test 4: Using repeat command (10 cyan sixels)"
    print -n $"\e[P1;1q"                    # Start Sixel
    print -n "#5;2;0;100;100"               # Cyan
    print -n "#5!10~"                       # Repeat '~' 10 times
    print -n $"\e\\"                        # End Sixel
    print ""
    print ""

    # Test 5: Carriage return test
    print "Test 5: Carriage return (should overwrite)"
    print -n $"\e[P1;1q"                    # Start Sixel
    print -n "#1;2;100;0;0"                 # Red
    print -n "#2;2;0;0;100"                 # Blue
    print -n "#1~~~"                        # Draw 3 red
    print -n "$"                            # Carriage return
    print -n "#2~"                          # Draw 1 blue (overwrites first red)
    print -n $"\e\\"                        # End Sixel
    print ""
    print ""

    print "=== All tests complete ==="
    print "If your terminal supports Sixel graphics, you should see colored bars above."
}

main
