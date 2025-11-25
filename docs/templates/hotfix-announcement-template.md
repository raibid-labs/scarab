# Hotfix Announcement Template

Use this template for urgent hotfix releases that address critical issues.

---

## Critical Issue Notice (Immediate)

**Post as soon as critical issue is confirmed**

### GitHub Issue Template

**Title**: 🚨 CRITICAL: [Brief description of issue]

**Labels**: `critical`, `bug`, `release-blocker`

**Body**:

```markdown
## Critical Issue Alert

**Affected Versions**: vX.Y.Z
**Severity**: [Critical / High]
**Impact**: [Data loss / Security vulnerability / Complete breakage / etc.]

### Description

[Clear, concise description of the problem]

### Who is Affected

- [ ] All users of vX.Y.Z
- [ ] Users with [specific configuration]
- [ ] Users on [specific platform]
- [ ] Users using [specific feature]

### Immediate Mitigation

**DO NOT** use vX.Y.Z until hotfix is released.

**Workaround** (if available):
```bash
# Steps to work around the issue
```

**Recommended Action**:
- If using vX.Y.Z: [Downgrade steps]
- If not yet upgraded: Stay on v[PREVIOUS]

### Timeline

- **Issue discovered**: [timestamp]
- **Fix in progress**: [ETA]
- **Hotfix release**: [ETA]

We will update this issue with progress every [2 hours / 4 hours].

### Technical Details

[For developers and advanced users]

[Root cause analysis if known]

---

**Next Update**: [timestamp]
```

---

### Social Media Alert

**Twitter/X** (Urgent):
```
🚨 CRITICAL: Do not use Scarab vX.Y.Z

Issue: [One-line description]
Impact: [Who's affected]

Hotfix coming in [timeframe]

Current users: [Mitigation steps]

Details: [link to issue]

We apologize for the inconvenience.
```

**Reddit Post** (r/rust, r/commandline):
```markdown
**[PSA] Critical Issue in Scarab vX.Y.Z - Do Not Use**

We've discovered a [critical/severe] issue in today's vX.Y.Z release:

**Issue**: [Description]
**Impact**: [What happens]
**Affected**: [Who]

**Action Required**:
- DO NOT upgrade to vX.Y.Z
- If already on vX.Y.Z: [Downgrade instructions]

**Status**: Hotfix in progress, ETA [timeframe]

We'll update this post as we make progress. Sorry for the disruption!

**Update thread**: [link to GitHub issue]
```

---

## Hotfix Release Announcement

**Post when hotfix is released**

### GitHub Release

**Title**: Scarab vX.Y.Z+1 (Emergency Hotfix)

**Mark as**: Latest Release (not pre-release)

**Body**:

```markdown
# Emergency Hotfix: vX.Y.Z+1

This is an emergency hotfix release addressing a critical issue in vX.Y.Z.

## Critical Fix

**Issue**: [Brief description]
**Severity**: [Critical/High]
**CVE** (if security): CVE-YYYY-XXXXX

### What Was Wrong

[Detailed explanation of the issue]

### Impact

[Who was affected and how]

- Affected versions: vX.Y.Z
- Affected platforms: [All / Specific platforms]
- Affected configurations: [All / Specific setups]

### The Fix

[Explanation of what was changed to fix the issue]

## Immediate Action Required

**If you are using vX.Y.Z, upgrade immediately:**

```bash
# macOS
brew upgrade scarab

# Arch Linux
yay -S scarab-terminal

# Cargo
cargo install --force scarab-client scarab-daemon

# Manual download
# Download binaries below
```

**If you have not yet upgraded to vX.Y.Z:**
- Skip vX.Y.Z entirely
- Upgrade directly to vX.Y.Z+1

## Changes

### Fixed
- **Critical**: [Description of fix] ([#issue](link))
- [Any other fixes included]

### Security
[If this is a security issue]

**CVE-YYYY-XXXXX** - [Description]
- **Severity**: [Critical/High/Medium/Low]
- **CVSS Score**: X.X
- **Vector**: [Attack vector]

**Credit**: Reported by [@username](link)

## Installation

[Standard installation instructions]

## Verification

Verify you're running the hotfix:

```bash
scarab-daemon --version
# Should output: scarab-daemon vX.Y.Z+1

scarab-client --version
# Should output: scarab-client vX.Y.Z+1
```

## Apology

We sincerely apologize for this issue and any disruption it caused. We are reviewing our release process to prevent similar issues in the future.

## Timeline

- **vX.Y.Z released**: [timestamp]
- **Issue discovered**: [timestamp]
- **Issue confirmed**: [timestamp]
- **Fix developed**: [timestamp]
- **vX.Y.Z+1 released**: [timestamp]

**Total time to fix**: [duration]

## Prevention

Steps we're taking to prevent this in the future:

- [Action item 1]
- [Action item 2]
- [Action item 3]

## Root Cause Analysis

[Link to post-mortem document or include brief summary]

## Support

If you continue to experience issues after upgrading:

1. Check [known issues](#known-issues)
2. [Open an issue](https://github.com/raibid-labs/scarab/issues/new)
3. Include:
   - Version: `scarab-daemon --version`
   - Platform: OS and architecture
   - Steps to reproduce

## Acknowledgments

Thanks to:
- [@reporter](link) for discovering and reporting the issue
- All contributors who helped with the rapid fix
- The community for patience and understanding

---

**Previous version vX.Y.Z has been marked as deprecated.**

**Full Diff**: [vX.Y.Z...vX.Y.Z+1](https://github.com/raibid-labs/scarab/compare/vX.Y.Z...vX.Y.Z+1)
```

---

### GitHub Discussions Post

**Title**: Hotfix vX.Y.Z+1 Released - Critical Issue Fixed

**Category**: Announcements

**Body**:

```markdown
# Hotfix Released: vX.Y.Z+1

The critical issue discovered in vX.Y.Z has been fixed.

## What Happened

[Non-technical explanation of the issue]

## The Fix

vX.Y.Z+1 is now available with the fix. Please upgrade immediately if you're on vX.Y.Z.

## How to Upgrade

```bash
# macOS
brew upgrade scarab

# Arch
yay -S scarab-terminal

# Cargo
cargo install --force scarab-client
```

[Download links]

## Verification

```bash
scarab-daemon --version
# Should show vX.Y.Z+1
```

## Status of vX.Y.Z

Version vX.Y.Z is deprecated and should not be used. We've:
- ✅ Released hotfix vX.Y.Z+1
- ✅ Yanked vX.Y.Z from crates.io
- ✅ Updated package managers
- ✅ Marked GitHub release as deprecated

## Our Apology

We apologize for any inconvenience. We're improving our testing to prevent this in the future.

## Questions?

Ask below or open an issue. We're here to help!

---

**Release Notes**: [Full details](link)
**Timeline**: [Incident timeline](link)
```

---

### Social Media Update

**Twitter/X Thread**:

```
1/4 ✅ Hotfix released: Scarab vX.Y.Z+1

The critical issue in vX.Y.Z is now fixed.

Upgrade immediately:
• brew upgrade scarab
• cargo install --force scarab-client

Release notes: [link]

2/4 What happened:

[Brief explanation of the issue and impact]

We caught this [timeframe] after release and worked immediately on a fix.

3/4 Prevention:

We're improving our [testing/CI/review process] to prevent similar issues:
• [Action 1]
• [Action 2]
• [Action 3]

4/4 Thank you for your patience and to everyone who reported the issue.

Special thanks to @reporter for the quick report!

We're committed to learning from this. 🙏

[Optional: Link to post-mortem when available]
```

**Reddit Update**:

```markdown
**[Update] Scarab vX.Y.Z+1 Hotfix Released**

Original issue: [link to PSA post]

The critical issue has been fixed in vX.Y.Z+1, released [timeframe] after the issue was discovered.

**Upgrade now:**
```bash
brew upgrade scarab  # macOS
yay -S scarab-terminal  # Arch
cargo install --force scarab-client  # Cargo
```

**What was fixed**: [Brief explanation]

**How we're preventing this**: [Key improvements]

Full release notes: [link]

Thanks for your patience, and apologies for the disruption!
```

---

## Internal Communication

### Team Notification (During Crisis)

**Subject**: CRITICAL: Scarab vX.Y.Z Issue

**Body**:

```
CRITICAL ISSUE DETECTED

Version: vX.Y.Z
Issue: [Description]
Severity: [Level]
Impact: [Who/what affected]

IMMEDIATE ACTIONS REQUIRED:

1. [Action for person/team]
2. [Action for person/team]
3. [Action for person/team]

STATUS UPDATES:
- Every 2 hours in #incidents channel
- Next update: [timestamp]

ROLES:
- Incident Lead: [Name]
- Technical Lead: [Name]
- Communications: [Name]

WAR ROOM: [Link/location for coordination]

All hands on deck until resolved.
```

---

### Post-Mortem Template

**Title**: Post-Mortem: vX.Y.Z Critical Issue

**Date**: YYYY-MM-DD

**Document**:

```markdown
# Post-Mortem: Scarab vX.Y.Z Critical Issue

## Summary

[2-3 sentence summary of what happened]

## Timeline

All times in [timezone]

| Time | Event |
|------|-------|
| [timestamp] | vX.Y.Z released |
| [timestamp] | Issue first reported by [user] |
| [timestamp] | Issue confirmed by team |
| [timestamp] | Investigation started |
| [timestamp] | Root cause identified |
| [timestamp] | Fix developed and tested |
| [timestamp] | vX.Y.Z+1 released |
| [timestamp] | All affected users notified |

**Total incident duration**: [X hours/days]
**Time to hotfix**: [X hours]

## Impact

- **Affected versions**: vX.Y.Z
- **Affected users**: [Estimate or "All users"]
- **Platforms**: [Specific or "All"]
- **Severity**: [Critical/High/Medium]

**User impact**:
- [What users experienced]
- [What operations were affected]
- [Data loss/corruption, if any]

**Downloads before hotfix**: [Number from GitHub/crates.io]

## Root Cause

### What Went Wrong

[Detailed technical explanation of the root cause]

### Why It Wasn't Caught

[Honest analysis of testing/review gaps]

- [ ] Missing test coverage
- [ ] Configuration not tested
- [ ] Platform-specific issue
- [ ] Edge case not considered
- [ ] Review process gap

## Resolution

### The Fix

[Technical description of the fix]

### Validation

[How we verified the fix]

## Action Items

### Immediate (Completed)

- [x] Hotfix released
- [x] Users notified
- [x] Documentation updated
- [x] Post-mortem scheduled

### Short-term (Next sprint)

- [ ] [Specific test to add]
- [ ] [Process improvement]
- [ ] [Documentation update]

**Owner**: [Name] | **Due**: [Date]

### Long-term (Next quarter)

- [ ] [Architectural improvement]
- [ ] [Tool/infrastructure]
- [ ] [Process change]

**Owner**: [Name] | **Due**: [Date]

## Lessons Learned

### What Went Well

- [Quick detection]
- [Fast response]
- [Good communication]

### What Could Be Improved

- [Testing coverage]
- [Review process]
- [Monitoring]

### What We'll Change

1. **[Change 1]**: [Description and rationale]
2. **[Change 2]**: [Description and rationale]
3. **[Change 3]**: [Description and rationale]

## Prevention

To prevent similar issues:

### Testing
- Add test case for [scenario]
- Expand [platform/configuration] testing
- Add [type] tests to CI

### Process
- [Review process improvement]
- [Release checklist update]
- [Documentation requirement]

### Infrastructure
- [Monitoring improvement]
- [Automation addition]
- [Tool implementation]

## Acknowledgments

- **Reporter**: [@user] for quick reporting
- **Responders**: [Team members who worked on fix]
- **Community**: For patience and understanding

## Conclusion

[Summary of what we learned and commitment to improvement]

---

**Review Date**: [1 month from incident]
**Reviewers**: [Team members]
```

---

## Communication Best Practices for Hotfixes

1. **Be transparent**: Explain what happened honestly
2. **Be timely**: Communicate early and often
3. **Be specific**: Clear about who's affected and what to do
4. **Be apologetic**: Take responsibility without making excuses
5. **Be forward-looking**: Explain prevention steps
6. **Be appreciative**: Thank reporters and patient users

## Hotfix Checklist

Quick checklist for hotfix process:

```
CRITICAL ISSUE RESPONSE:
- [ ] Confirm severity and impact
- [ ] Create critical issue on GitHub
- [ ] Notify team immediately
- [ ] Post PSA on social media
- [ ] Start fix development

HOTFIX DEVELOPMENT:
- [ ] Create hotfix branch from tag
- [ ] Develop and test fix
- [ ] Update CHANGELOG
- [ ] Bump patch version
- [ ] Internal review

HOTFIX RELEASE:
- [ ] Tag and push hotfix
- [ ] Monitor CI/CD
- [ ] Publish release
- [ ] Yank old version from crates.io
- [ ] Update package managers

COMMUNICATION:
- [ ] Update critical issue
- [ ] Post release announcement
- [ ] Social media updates
- [ ] Email affected users (if known)
- [ ] Update documentation

POST-INCIDENT:
- [ ] Schedule post-mortem
- [ ] Document lessons learned
- [ ] Create prevention action items
- [ ] Update release process
```
