# Claude Bootstrap Prompt

Use the following prompt to bootstrap Claude CLI for the next phase of work:

```text
I have completed a technical audit of the Scarab codebase and identified the next steps for integrating the 'mimic' testing library. 

Please read the following files to context yourself:
1. docs/AUDIT_REPORT.md (The full audit findings)
2. docs/MIMIC_ISSUES.md (The feature requests needed for the mimic library)
3. crates/scarab-daemon/tests/vte_conformance.rs (The current incomplete test)

Your goal is to help me implement the "Action Items" listed in the Audit Report. Specifically:
1. We need to modify the `mimic` library (assumed to be at `../mimic` relative to this repo, or we mock it if inaccessible) to add the requested APIs.
2. Update `vte_conformance.rs` to use these new APIs for true verification.

Start by acknowledging the audit findings and asking if you should proceed with modifying the local `mimic` crate (if available) or if we need to simulate the changes.
```
