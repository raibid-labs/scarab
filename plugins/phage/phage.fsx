// Phage Plugin for Scarab Terminal
// Provides Phage context management capabilities via the Dock & Menu system

// Plugin metadata
let name = "phage"
let version = "0.1.0"
let description = "Phage AI context management integration"
let icon = "🦠"

// Phage daemon configuration
let daemon_url = "http://localhost:15702"

// Menu items for the dock
let menu = [
    { label = "Init Workspace"; action = "init_cmd"; icon = "🌱"; shortcut = "Ctrl+Alt+I" };
    { label = "Chat"; action = "chat_open"; icon = "💬"; shortcut = "Ctrl+Alt+C" };
    { label = "Explain Selection"; action = "explain_sel"; icon = "🧐"; shortcut = "" };
    { label = "Fix Last Command"; action = "fix_cmd"; icon = "🔧"; shortcut = "" };
    { label = "Context Info"; action = "show_context"; icon = "📋"; shortcut = "" }
]

// Initialize Phage workspace in current directory
let init_workspace cwd =
    let phage_dir = cwd + "/.phage"

    // Check if already initialized
    if file_exists phage_dir then
        log_warn "Phage workspace already initialized at " + phage_dir
        false
    else
        // Create directory structure
        create_dir (phage_dir + "/layers/base")
        create_dir (phage_dir + "/layers/project")
        create_dir (phage_dir + "/layers/session")

        // Create base config
        let base_config = "{
    name = \"base-default\"
    rules = []
}"
        write_file (phage_dir + "/layers/base/config.toml") base_config

        // Create project config
        let project_config = "{
    name = \"project-default\"
    extends = \"base-default\"
    rules = []
}"
        write_file (phage_dir + "/layers/project/config.toml") project_config

        // Create session config
        let session_config = "{
    name = \"session\"
    extends = \"project-default\"
    volatile = true
}"
        write_file (phage_dir + "/layers/session/config.toml") session_config

        // Create workspace metadata
        let metadata = "{
    version = \"1.0\"
    created = \"" + timestamp_now () + "\"
}"
        write_file (phage_dir + "/workspace.toml") metadata

        // Create .gitignore
        write_file (phage_dir + "/.gitignore") "session/\n*.log\n"

        log_info "Phage workspace initialized at " + phage_dir
        true

// Plugin lifecycle hooks
let on_load ctx =
    log_info "🦠 Phage plugin loaded"
    log_info "   Daemon URL: " + daemon_url
    ()

let on_unload () =
    log_info "🦠 Phage plugin unloaded"
    ()

// Handle menu action commands
let on_remote_command id ctx =
    match id with
    | "init_cmd" ->
        let cwd = get_cwd ctx
        if init_workspace cwd then
            notify_success ctx "Phage Initialized" "Workspace created successfully"
        else
            notify_warn ctx "Already Initialized" "Phage workspace already exists"
        ()
    | "chat_open" ->
        log_info "🦠 Opening Phage chat interface..."
        notify_info ctx "Chat" "Chat interface not yet implemented"
        ()
    | "explain_sel" ->
        let selection = get_selection ctx
        if selection = "" then
            notify_warn ctx "No Selection" "Please select text to explain"
        else
            log_info "🦠 Explaining selection: " + selection
            notify_info ctx "Explain" "Selection explanation not yet implemented"
        ()
    | "fix_cmd" ->
        let last_cmd = get_last_failed_command ctx
        if last_cmd = "" then
            notify_info ctx "No Failed Command" "No failed command in history"
        else
            log_info "🦠 Fixing command: " + last_cmd
            notify_info ctx "Fix Command" "Command fix not yet implemented"
        ()
    | "show_context" ->
        log_info "🦠 Showing context info..."
        notify_info ctx "Context" "Context info not yet implemented"
        ()
    | _ ->
        log_warn "Unknown command: " + id
        ()
