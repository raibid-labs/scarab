# Plugin Security Architecture

Security is a critical concern when allowing third-party code to run in your terminal. Scarab implements a comprehensive security model to protect users while enabling powerful plugin capabilities.

## Overview

Scarab's security model is built on these principles:

1. **Sandboxing** - Plugins run in isolated environments with limited capabilities
2. **Permissions** - Explicit opt-in for sensitive operations
3. **Trust Levels** - Different levels of trust for different plugin sources
4. **Transparency** - Users always know what plugins can access
5. **Defense in Depth** - Multiple layers of protection

## Plugin Sandboxing

### What Plugins Cannot Access

Regardless of permissions, plugins are **never** allowed to:

- Execute arbitrary system commands
- Load native libraries or FFI
- Access memory outside their sandbox
- Modify the Scarab binary or configuration
- Access other plugins' data
- Bypass permission checks
- Escalate their privilege level

### What Plugins Can Access (By Default)

Without any permissions requested, plugins can:

- Read terminal output and input (backend only)
- Log messages
- Store plugin-specific data with `ctx.SetData/GetData`
- Show notifications and UI overlays
- Access environment variables passed to the terminal
- Read terminal dimensions and cursor position
- Read terminal cell contents

### Resource Limits

Each plugin operates under strict resource limits:

**CPU Limits:**
- Frontend: 16ms per frame (or dropped frames)
- Backend: 50μs per output line (or throttled)
- Maximum CPU time per hook: 100ms

**Memory Limits:**
- Frontend: 50MB per plugin
- Backend: 100MB per plugin
- Automatic termination if exceeded

**File System Limits:**
- Read-only access to plugin directory
- Write access to plugin data directory only
- Maximum 10MB of stored data per plugin

**Network Restrictions:**
- Outbound HTTP/HTTPS only (no raw sockets)
- Maximum 100 requests/minute
- Total bandwidth: 10MB/minute

Plugins that exceed limits are automatically throttled or terminated.

## Permission System

### Available Permissions

Plugins must request permissions in `plugin.toml` for sensitive operations:

#### `network.http`
Access external HTTP/HTTPS APIs.

```toml
[permissions]
network.http = { domains = ["api.github.com", "example.com"], reason = "Fetch GitHub PR data" }
```

**Grants access to:**
- Make HTTP GET/POST requests to specified domains
- Read response data

**Restrictions:**
- Domain whitelist enforced
- No access to local network (localhost, 192.168.*, etc.)
- HTTPS enforced for sensitive data

#### `filesystem.read`
Read files from specific directories.

```toml
[permissions]
filesystem.read = { paths = ["~/.gitconfig", "~/.ssh/config"], reason = "Parse Git configuration" }
```

**Grants access to:**
- Read files at specified paths
- List directory contents

**Restrictions:**
- Glob patterns supported but must be explicit
- No access to `/etc`, `/sys`, `/proc` by default
- No access to other users' home directories

#### `filesystem.write`
Write files to specific directories.

```toml
[permissions]
filesystem.write = { paths = ["~/.scarab/plugins/{plugin_name}/cache"], reason = "Cache API responses" }
```

**Grants access to:**
- Create and write files
- Create directories

**Restrictions:**
- More restrictive than read permissions
- Cannot write outside plugin data directory without explicit approval
- Automatic size limits enforced

#### `clipboard.read`
Read from system clipboard.

```toml
[permissions]
clipboard.read = { reason = "Implement clipboard history" }
```

**Grants access to:**
- Read current clipboard contents
- Monitor clipboard changes

**Security notes:**
- Can access sensitive data (passwords, API keys)
- Users see warning during installation

#### `clipboard.write`
Write to system clipboard.

```toml
[permissions]
clipboard.write = { reason = "Copy URLs to clipboard" }
```

**Grants access to:**
- Modify clipboard contents

**Security notes:**
- Less sensitive than read
- Could interfere with user workflow

#### `environment.read`
Read environment variables beyond basic terminal variables.

```toml
[permissions]
environment.read = { vars = ["AWS_ACCESS_KEY_ID", "GITHUB_TOKEN"], reason = "Authenticate with cloud services" }
```

**Grants access to:**
- Read specified environment variables
- Access to sensitive credentials

**Security notes:**
- HIGH RISK permission
- Users see strong warning
- Audited plugins only

#### `pty.write`
Inject data into the PTY (simulate user input).

```toml
[permissions]
pty.write = { reason = "Implement command shortcuts" }
```

**Grants access to:**
- Send keystrokes to PTY
- Execute commands as user

**Security notes:**
- CRITICAL permission
- Can execute arbitrary commands
- Requires user review and consent

### Requesting Permissions

Add permissions to `plugin.toml`:

```toml
[plugin]
name = "github-integration"
version = "0.1.0"

[permissions]
network.http = {
    domains = ["api.github.com"],
    reason = "Fetch pull request information"
}
filesystem.read = {
    paths = ["~/.gitconfig"],
    reason = "Read Git user configuration"
}
```

**Best practices:**
- Only request permissions you actually need
- Provide clear, specific reasons
- Use minimal scope (specific domains, not wildcards)
- Document permission usage in README

### User Consent Flow

When a user installs a plugin with permissions:

1. **Permission Review Screen**
   - Lists all requested permissions
   - Shows reason for each permission
   - Displays plugin source and trust level

2. **User Decision**
   - Approve all permissions
   - Deny installation
   - Approve with restrictions (advanced)

3. **Runtime Enforcement**
   - Scarab enforces approved permissions
   - Violations logged and reported
   - Plugin disabled on repeated violations

**Example consent screen:**

```
Installing: github-integration v0.1.0
Source: Official Scarab Registry (Trusted)

Requested Permissions:
  [!] network.http
      Domains: api.github.com
      Reason: Fetch pull request information
      Risk: Medium - Can send data to external server

  [!] filesystem.read
      Paths: ~/.gitconfig
      Reason: Read Git user configuration
      Risk: Low - Read-only access to config

Continue installation? [Y/n]
```

## Trust Levels

Scarab categorizes plugins into trust levels based on their source:

### Level 1: Official Plugins
**Source:** Bundled with Scarab or official repository

**Characteristics:**
- Audited by Scarab maintainers
- Signed with official key
- Automatic updates enabled
- Minimal permission warnings

**Examples:**
- git-status
- command-timer
- clipboard-history

### Level 2: Verified Authors
**Source:** Known, verified plugin developers

**Characteristics:**
- Author verified via GitHub/email
- Code review by community
- Displayed with verified badge
- Standard permission warnings

**Requirements:**
- Public source code repository
- Clear authorship and contact info
- History of safe plugins

### Level 3: Community Plugins
**Source:** Public plugin registry, unverified authors

**Characteristics:**
- No formal verification
- Community ratings/reviews
- Stronger permission warnings
- Manual update approval

**Warnings:**
- "This plugin is from an unverified author"
- Permission requests shown prominently
- Source code review recommended

### Level 4: Local Development
**Source:** Local filesystem, development mode

**Characteristics:**
- Not signed
- Full permissions available (with warnings)
- Not distributed
- Development-only

**Use cases:**
- Plugin development
- Private/internal plugins
- Testing and debugging

### Signature Verification

Official and verified plugins are cryptographically signed:

```toml
[signature]
algorithm = "ed25519"
public_key = "AGE4d3V2..."
signature = "oWmN8F2p..."
```

Scarab verifies signatures before loading plugins:

1. Check signature matches plugin content
2. Verify public key against trust store
3. Confirm author identity
4. Allow/deny based on trust level

Unsigned plugins from official sources are rejected.

## Best Practices for Plugin Authors

### 1. Principle of Least Privilege

Request only the minimum permissions needed:

**Bad:**
```toml
[permissions]
filesystem.read = { paths = ["~/**"], reason = "Need file access" }
```

**Good:**
```toml
[permissions]
filesystem.read = { paths = ["~/.gitconfig"], reason = "Parse Git user email" }
```

### 2. Secure API Key Handling

Never hardcode secrets:

**Bad:**
```fsharp
let apiKey = "sk-1234567890abcdef"  // NEVER DO THIS
```

**Good:**
```fsharp
// Read from user's environment
match ctx.GetEnv "GITHUB_TOKEN" with
| Some token -> useToken token
| None -> ctx.NotifyError "Setup Required" "Please set GITHUB_TOKEN environment variable"
```

**Best:**
```fsharp
// Use config.toml for per-user secrets
match ctx.Config.GetOpt "api_key" with
| Some key -> useApiKey key
| None -> ctx.NotifyError "Setup Required" "Please add api_key to config.toml"
```

### 3. Input Validation

Always validate untrusted input:

```fsharp
[<OnOutput>]
let onOutput ctx line =
    async {
        // Validate before processing
        if line.Length > 10000 then
            ctx.Log Warn "Line too long, skipping"
            return Continue

        // Sanitize before external API calls
        let sanitized = sanitizeForApi line
        let! result = callExternalApi sanitized

        return Continue
    }
```

### 4. Output Sanitization

Sanitize data before displaying to users:

```fsharp
let sanitizeHtml (text: string) =
    text
        .Replace("&", "&amp;")
        .Replace("<", "&lt;")
        .Replace(">", "&gt;")
        .Replace("\"", "&quot;")

let showNotification ctx message =
    let safe = sanitizeHtml message
    ctx.Notify "API Response" safe Info
```

### 5. Rate Limiting

Implement rate limiting for external services:

```fsharp
let mutable lastRequest = DateTime.MinValue
let rateLimitMs = 1000  // Max 1 request/second

[<OnOutput>]
let onOutput ctx line =
    async {
        if (DateTime.Now - lastRequest).TotalMilliseconds < rateLimitMs then
            return Continue  // Skip, too soon

        lastRequest <- DateTime.Now
        // Make API call...
        return Continue
    }
```

### 6. Error Handling

Never expose sensitive info in errors:

**Bad:**
```fsharp
try
    authenticateWithToken apiKey
with
| ex -> ctx.NotifyError "Auth Failed" ex.Message  // May expose secrets!
```

**Good:**
```fsharp
try
    authenticateWithToken apiKey
with
| :? UnauthorizedException ->
    ctx.NotifyError "Auth Failed" "Invalid API key"
| ex ->
    ctx.Log Error (sprintf "Unexpected error: %s" ex.Message)
    ctx.NotifyError "Error" "An unexpected error occurred"
```

### 7. Secure Defaults

Default to secure configurations:

```fsharp
let getConfig (ctx: PluginContext) =
    {
        EnableTelemetry = ctx.Config.GetOpt "telemetry" |> Option.defaultValue false  // Opt-in
        AllowExternal = ctx.Config.GetOpt "allow_external" |> Option.defaultValue false
        VerifySSL = ctx.Config.GetOpt "verify_ssl" |> Option.defaultValue true  // Always default true
    }
```

### 8. Audit Logging

Log security-relevant events:

```fsharp
// Log permission usage
ctx.Log Info "Reading file: ~/.gitconfig"
let content = readFile "~/.gitconfig"

// Log external API calls
ctx.Log Info (sprintf "Calling API: %s" endpoint)
let! response = httpGet endpoint

// Log authentication attempts
ctx.Log Info "Authenticating with GitHub"
```

## Security Incident Response

### Reporting Vulnerabilities

If you discover a security vulnerability:

1. **Do NOT open a public issue**
2. Email security@scarab-terminal.com with:
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact
   - Suggested fix (if any)
3. Wait for acknowledgment (within 48 hours)
4. Allow 90 days for fix before public disclosure

### Plugin Security Issues

If you find a malicious plugin:

1. Report to security@scarab-terminal.com
2. Include:
   - Plugin name and version
   - What it's doing wrong
   - Evidence (logs, network traces)
3. The plugin will be:
   - Immediately removed from registry
   - Signature revoked
   - Users notified to uninstall

### For Plugin Users

If you suspect a plugin is malicious:

1. Disable it immediately: `scarab plugin disable <name>`
2. Report to security@scarab-terminal.com
3. Review logs: `scarab logs --plugin <name>`
4. Check network activity: `scarab plugin inspect <name>`

## Security Checklist for Plugin Authors

Before publishing a plugin:

- [ ] Only request necessary permissions
- [ ] Provide clear, specific permission reasons
- [ ] Never hardcode secrets or API keys
- [ ] Validate all untrusted input
- [ ] Sanitize all output
- [ ] Implement rate limiting for external APIs
- [ ] Handle errors without exposing sensitive data
- [ ] Use secure defaults in configuration
- [ ] Log security-relevant operations
- [ ] Test with minimal permissions
- [ ] Document security considerations in README
- [ ] Keep dependencies updated
- [ ] Respond to security reports promptly

## Security Checklist for Plugin Users

Before installing a plugin:

- [ ] Check the plugin source (official/verified/community)
- [ ] Review requested permissions
- [ ] Read permission reasons carefully
- [ ] Check plugin ratings and reviews
- [ ] Review source code for critical permissions
- [ ] Look for recent updates (maintained?)
- [ ] Check author reputation
- [ ] Start with minimal permissions if possible
- [ ] Monitor plugin behavior after install
- [ ] Report suspicious activity

## Advanced Security Features

### Content Security Policy (CSP)

Plugins can declare a CSP for RemoteUI components:

```toml
[security]
csp = "default-src 'self'; script-src 'none'; connect-src https://api.github.com"
```

Prevents XSS and other injection attacks in UI components.

### Capability-Based Security

Fine-grained permission model:

```toml
[permissions.network.http]
domains = ["api.github.com"]
methods = ["GET"]  # Only GET, no POST/PUT/DELETE
max_requests_per_minute = 60
max_response_size = "1MB"
```

### Audit Mode

Enable verbose security logging:

```bash
scarab plugin install my-plugin --audit-mode
```

Logs all permission checks and API calls for review.

### Permission Revocation

Remove permissions after installation:

```bash
scarab plugin revoke my-plugin network.http
```

Plugin continues running but API calls fail gracefully.

## Security FAQ

**Q: Can plugins access my passwords?**
A: Only if they have `clipboard.read` permission AND you copy a password. Always review clipboard permissions carefully.

**Q: Can plugins execute commands?**
A: Only if they have `pty.write` permission. This is a critical permission that requires explicit user consent.

**Q: Can plugins read my files?**
A: Only specific files listed in `filesystem.read` permission. Users approve each path during installation.

**Q: Can plugins send my data to external servers?**
A: Only if they have `network.http` permission. Check which domains are allowed.

**Q: Can I trust official plugins?**
A: Official plugins are audited by Scarab maintainers and signed cryptographically. They're as trustworthy as Scarab itself.

**Q: How do I audit a plugin before installing?**
A: Review source code, check requested permissions, read reviews, and install in audit mode for detailed logging.

**Q: What happens if I deny permissions?**
A: Plugin installation fails. You cannot install a plugin without approving its permissions.

**Q: Can plugins update their permissions?**
A: Yes, but users must approve new permissions before the update is installed.

## Next Steps

- **[Plugin Lifecycle](plugin-lifecycle.md)** - How plugins load and run
- **[Performance Guide](performance.md)** - Optimization strategies
- **[API Reference: PluginContext](../api-reference/plugin-context.md)** - Available APIs
- **[Tutorial 7: Testing and Publishing](../tutorials/07-testing-and-publishing.md)** - Production readiness

## Getting Help

- Security questions: security@scarab-terminal.com
- Plugin development: GitHub Discussions
- Bug reports: GitHub Issues
- Emergency security issues: security@scarab-terminal.com (GPG key available)

---

**Remember:** Security is everyone's responsibility. When in doubt, request fewer permissions, validate more input, and document your security considerations.
