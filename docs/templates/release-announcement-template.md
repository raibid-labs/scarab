# Release Announcement Template

Use this template for announcing releases on GitHub Discussions, social media, and community channels.

---

## GitHub Discussions Post

**Title**: Scarab vX.Y.Z Released - [Brief highlight]

**Category**: Announcements

**Body**:

```markdown
# Scarab vX.Y.Z is here! 🎉

[Opening paragraph - what's exciting about this release?]

## Highlights

[3-5 key highlights with brief descriptions]

### 🚀 [Feature Name]

[1-2 sentences describing the feature and its benefit]

```bash
# Example usage
scarab-daemon --new-feature
```

### 🎨 [Feature Name]

[1-2 sentences describing the feature and its benefit]

### 🐛 Bug Fixes

[Summary of important bug fixes]

## Download

Get the latest release:

- **macOS**: `brew upgrade scarab`
- **Arch Linux**: `yay -S scarab-terminal`
- **Cargo**: `cargo install --force scarab-client`
- **Binaries**: [Download page](https://github.com/raibid-labs/scarab/releases/tag/vX.Y.Z)

## Full Details

See the [release notes](https://github.com/raibid-labs/scarab/releases/tag/vX.Y.Z) for complete changelog and upgrade instructions.

## What's Next

[Brief mention of what's coming in next release]

See our [ROADMAP.md](https://github.com/raibid-labs/scarab/blob/main/ROADMAP.md) for the full development plan.

## Feedback Welcome

We'd love to hear your thoughts! Report issues or share feedback:
- [GitHub Issues](https://github.com/raibid-labs/scarab/issues)
- [Discussions](https://github.com/raibid-labs/scarab/discussions)

Thanks to everyone who contributed to this release! 🙏
```

---

## Twitter/X Post

**Thread Structure** (280 chars per tweet):

### Tweet 1 (Main Announcement)
```
🎉 Scarab vX.Y.Z is out!

[One-line summary of biggest feature/improvement]

Download: [link]
Release notes: [link]

#RustLang #Terminal #OpenSource
```

### Tweet 2 (Key Features)
```
✨ What's new:

• [Feature 1 - one line]
• [Feature 2 - one line]
• [Feature 3 - one line]
• [X bug fixes and improvements]

[Optional: Screenshot or demo GIF]
```

### Tweet 3 (Call to Action)
```
Try it now:

macOS: brew install scarab
Linux: cargo install scarab-client
Arch: yay -S scarab-terminal

[Optional: Quick start command or demo]
```

### Tweet 4 (Thanks)
```
Big thanks to our contributors:

@contributor1 @contributor2 @contributor3

And everyone who reported issues and provided feedback! 🙏

[Link to full contributor list]
```

---

## Reddit Post

### r/rust

**Title**: [Announcing] Scarab vX.Y.Z - [Brief highlight]

**Body**:

```markdown
Hi r/rust!

I'm excited to announce Scarab vX.Y.Z, a GPU-accelerated terminal emulator with a unique plugin system using Fusabi (F# for Rust).

## What is Scarab?

Scarab is a split-process terminal emulator built in Rust with:
- **Bevy-based rendering** for smooth 60+ FPS performance
- **Zero-copy IPC** using shared memory
- **Hybrid plugin system** with Fusabi-lang (F# dialect)
- **Cross-platform** support (Linux, macOS, Windows experimental)

## What's New in vX.Y.Z

[3-5 key highlights with more detail than Twitter]

### [Feature Name]

[1-2 paragraphs describing the feature, why it matters, and how to use it]

```rust
// Code example if relevant
```

## Installation

```bash
# macOS
brew tap raibid-labs/scarab
brew install scarab

# Arch Linux
yay -S scarab-terminal

# From source
cargo install scarab-client scarab-daemon
```

## Links

- [Release Notes](link)
- [GitHub Repo](link)
- [Documentation](link)

## Feedback

We're actively developing Scarab and would love your feedback! Feel free to:
- Try it out and report issues
- Suggest features
- Contribute (we're beginner-friendly!)

Thanks for reading!
```

---

### r/commandline

**Title**: Scarab vX.Y.Z - GPU-accelerated terminal with F# plugins

**Body**:

```markdown
Scarab vX.Y.Z just dropped!

**TL;DR**: Fast terminal emulator, Bevy graphics, F# scripting

## Quick Highlights

- [Feature in user-benefit terms]
- [Feature in user-benefit terms]
- [Feature in user-benefit terms]

## Install

macOS: `brew install scarab`
Arch: `yay -S scarab-terminal`
Others: `cargo install scarab-client`

Full release notes: [link]

[Optional: Screenshot]

Feedback and contributions welcome!
```

---

## Mastodon Post

**Character limit**: 500

```markdown
🎉 Scarab vX.Y.Z released!

[Summary of key features]

New:
• [Feature 1]
• [Feature 2]
• [Feature 3]
+ [X] bug fixes

Built with #RustLang, powered by @bevyengine

Download: [link]
Release notes: [link]

#Terminal #OpenSource #GameDev
```

---

## Discord/Slack Announcement

**#announcements channel**:

```markdown
@everyone Scarab vX.Y.Z is here! 🎉

**What's new:**
• [Feature 1]
• [Feature 2]
• [Feature 3]

**Download:** https://github.com/raibid-labs/scarab/releases/tag/vX.Y.Z

**Highlights:**

[1-2 paragraphs about the most exciting changes]

**Try it:**
\`\`\`bash
# macOS
brew upgrade scarab

# Cargo
cargo install --force scarab-client
\`\`\`

**Help us test!** Report any issues in #bug-reports

Thanks to everyone who contributed! Full release notes: [link]
```

---

## Email Newsletter

**Subject**: Scarab vX.Y.Z: [Exciting feature/improvement]

**Body**:

```html
<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8">
  <title>Scarab vX.Y.Z Released</title>
</head>
<body style="font-family: sans-serif; max-width: 600px; margin: 0 auto; padding: 20px;">

  <h1>Scarab vX.Y.Z is Here!</h1>

  <p>[Opening paragraph - what's exciting?]</p>

  <h2>Highlights</h2>

  <ul>
    <li><strong>[Feature Name]</strong>: [Description]</li>
    <li><strong>[Feature Name]</strong>: [Description]</li>
    <li><strong>[Feature Name]</strong>: [Description]</li>
  </ul>

  <h2>Get Started</h2>

  <p>Install or upgrade Scarab:</p>

  <pre style="background: #f5f5f5; padding: 10px; border-radius: 5px;">
# macOS
brew upgrade scarab

# Arch Linux
yay -S scarab-terminal

# Cargo
cargo install --force scarab-client
  </pre>

  <p>
    <a href="https://github.com/raibid-labs/scarab/releases/tag/vX.Y.Z"
       style="background: #0366d6; color: white; padding: 10px 20px;
              text-decoration: none; border-radius: 5px; display: inline-block;">
      Download Binaries
    </a>
  </p>

  <h2>What's Next</h2>

  <p>[Brief preview of upcoming features]</p>

  <hr style="margin: 30px 0;">

  <p style="font-size: 14px; color: #666;">
    <a href="https://github.com/raibid-labs/scarab">GitHub</a> •
    <a href="https://github.com/raibid-labs/scarab/blob/main/CHANGELOG.md">Changelog</a> •
    <a href="https://github.com/raibid-labs/scarab/discussions">Discussions</a>
  </p>

</body>
</html>
```

---

## Blog Post Outline

**Title**: Scarab vX.Y.Z: [Compelling headline about main feature]

**Structure**:

1. **Introduction** (2-3 paragraphs)
   - What's new in this release
   - Why it matters
   - Who benefits

2. **Feature Deep-Dive** (3-4 sections)
   - Each major feature gets its own section
   - Include code examples, screenshots, or demos
   - Explain technical details and trade-offs

3. **Performance Improvements** (if applicable)
   - Benchmarks and comparisons
   - Real-world impact

4. **Breaking Changes** (if any)
   - What changed and why
   - Migration guide
   - Code examples

5. **Community Highlights**
   - Contributor shout-outs
   - Interesting use cases
   - Community feedback

6. **What's Next**
   - Preview of upcoming features
   - Roadmap link
   - Call for contributions

7. **Get Started**
   - Installation instructions
   - Quick start guide
   - Links to resources

---

## Community Response Template

**For first community comments/feedback**:

```markdown
Thanks for trying Scarab vX.Y.Z!

[If reporting issue]:
Appreciate the bug report! I've created an issue to track this: #[number]

[If sharing feedback]:
Great to hear [specific point]! [Response to their feedback]

[If asking question]:
[Answer their question with relevant links]

Let us know if you run into any other issues or have suggestions!
```

---

## Post-Release Social Media Follow-ups

**Day 1**: Main announcement
**Day 2**: Feature spotlight with demo/screenshot
**Day 3**: Community reaction/feedback compilation
**Day 7**: Week one stats (downloads, stars, issues)
**Day 30**: One month retrospective (if major release)

---

## Tips for Effective Announcements

1. **Lead with value**: Focus on what users gain, not technical details
2. **Use visuals**: Screenshots, GIFs, videos increase engagement
3. **Keep it concise**: Respect readers' time, link to details
4. **Call to action**: Make it clear what you want readers to do
5. **Thank contributors**: Build community by recognition
6. **Cross-post strategically**: Tailor message to each platform
7. **Engage with comments**: Respond to feedback promptly
8. **Share user content**: Retweet/share community reactions

## Metrics to Track

- GitHub release page views
- Download counts by platform
- Social media engagement (likes, shares, comments)
- New issues/discussions created
- Crates.io download stats
- Community sentiment (positive/negative feedback ratio)
