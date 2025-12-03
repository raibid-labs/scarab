#!/usr/bin/env nu
# Scarab Deep Shell Integration Demo for Nushell
#
# This script demonstrates the OSC 133 shell integration markers
# and semantic zones functionality in Scarab terminal.
#
# For Nushell, add these hooks to your config.nu:
#
# $env.config.hooks = {
#     pre_prompt: [{|| print -n $"\e]133;D;($env.LAST_EXIT_CODE)\e\\" }]
#     pre_execution: [{|| print -n $"\e]133;C\e\\" }]
#     env_change: {
#         PWD: [{|before, after| print -n $"\e]133;A\e\\" }]
#     }
# }
#
# And set your prompt to emit B marker:
#
# $env.PROMPT_COMMAND = {||
#     # Your prompt here
#     $"(ansi green)($env.PWD)(ansi reset) > "
# }
# $env.PROMPT_COMMAND_RIGHT = ""
# $env.PROMPT_INDICATOR = {|| print -n $"\e]133;B\e\\"; "" }

def main [] {
    print "═══════════════════════════════════════════════════════"
    print "  Scarab Deep Shell Integration Demo (Nushell)"
    print "═══════════════════════════════════════════════════════"
    print ""
    print "This demo shows OSC 133 marker sequences for Nushell."
    print ""
    print "Features enabled with proper config:"
    print "  * Semantic zones (Prompt / Input / Output)"
    print "  * Command duration tracking"
    print "  * Exit code indicators (green checkmark / red X)"
    print "  * Copy last output (Ctrl+Shift+Y)"
    print ""

    # Emit example sequences
    print "Example OSC 133 sequences:"
    print ""

    print "  Prompt start (A marker):"
    print $"    ESC ] 133 ; A ESC \\"
    print -n $"\e]133;A\e\\"
    print ""

    print "  Command start (B marker):"
    print $"    ESC ] 133 ; B ESC \\"
    print -n $"\e]133;B\e\\"
    print ""

    print "  Output start (C marker):"
    print $"    ESC ] 133 ; C ESC \\"
    print -n $"\e]133;C\e\\"
    print ""

    print "  Command done with exit code 0 (D marker):"
    print $"    ESC ] 133 ; D ; 0 ESC \\"
    print -n $"\e]133;D;0\e\\"
    print ""
    print ""

    print "═══════════════════════════════════════════════════════"
    print "  Nushell Configuration"
    print "═══════════════════════════════════════════════════════"
    print ""
    print "Add this to your config.nu to enable shell integration:"
    print ""
    print "```nushell"
    print '$env.config.hooks = {'
    print '    pre_prompt: [{||'
    print '        let code = if ($env.LAST_EXIT_CODE? | is-empty) { 0 } else { $env.LAST_EXIT_CODE }'
    print '        print -n $"\e]133;D;($code)\e\\"'
    print '        print -n $"\e]133;A\e\\"'
    print '    }]'
    print '    pre_execution: [{|| print -n $"\e]133;C\e\\" }]'
    print '}'
    print ''
    print '$env.PROMPT_INDICATOR = {||'
    print '    print -n $"\e]133;B\e\\"'
    print '    "> "'
    print '}'
    print "```"
    print ""
    print "═══════════════════════════════════════════════════════"
}

# Demo function that emits markers manually for testing
def "main demo" [] {
    print ""
    print "Running demo sequence with markers..."
    print ""

    # Simulate prompt
    print -n $"\e]133;A\e\\"
    print "prompt> "

    # Simulate command start
    print -n $"\e]133;B\e\\"
    print "echo 'Hello Scarab'"

    # Simulate output start
    print -n $"\e]133;C\e\\"
    print "Hello Scarab"

    # Simulate command done (success)
    print -n $"\e]133;D;0\e\\"
    print ""

    # Simulate failed command
    print -n $"\e]133;A\e\\"
    print "prompt> "
    print -n $"\e]133;B\e\\"
    print "ls /nonexistent"
    print -n $"\e]133;C\e\\"
    print "ls: cannot access '/nonexistent': No such file or directory"
    print -n $"\e]133;D;2\e\\"
    print ""

    print ""
    print "Demo complete! Check Scarab gutter for zone indicators."
    print "Press Ctrl+Shift+Y to copy the last output."
}

main
