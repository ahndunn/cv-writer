---
name: cv-writer-scrum-team
description: >-
  Orchestrates the Scrum Team workflow (PO, BA, PM, SA, Tech Lead, Developer, QC)
  for building and maintaining the stateless Rust LuaLaTeX CV Writer CLI tool.
---

# CV Writer Scrum Team Skill

This skill defines the operational workflow, role checklists, and review gates for the CV Writer CLI project.

## Operational Lifecycle

1. **Sprint Planning & Vision (PO + PM)**:
   - Establish sprint goals.
   - Guard statelessness, single-invoke Docker portability, and zero-state constraints.

2. **Analysis & Schema Definition (BA + PO)**:
   - Define canonical resume schema (Contact, Education, Experience, Skills, Certifications, Publications, Awards, Projects).
   - Ensure AI Agent friendliness with self-descriptive field schemas and changelog diff tracking.

3. **Architectural & Modernization Review (SA + Tech Lead)**:
   - Template migration to modern LuaLaTeX standards (OpenType fonts via `fontspec`, utf-8 native handling).
   - Rust architecture: Agent-first CLI (`clap`), template renderer (`tera`), changelog engine (`similar`), isolated ephemeral compilation in `/tmp` ramfs with automatic sweep.

4. **Implementation (Developer)**:
   - Modernized `.tex` template with Tera tokens.
   - Rust crates: `clap`, `schemars`, `serde`, `serde_json`, `tokio`, `tera`, `tempfile`, `similar`, `chrono`.
   - CLI Subcommands:
     - `schema`: Returns the full JSON schema for an AI Agent to inspect.
     - `sample`: Returns example valid input data matching Star Rover layout.
     - `template-info`: Returns typographical and template specifications.
     - `compile`: Accepts structured profile data, validates, compiles via LuaLaTeX, optionally records changelog diffs, returns exit codes and concise diagnostics.
   - Production Dockerfile with TeXLive LuaLaTeX + Rust binary.
   - **Quality Gate**: Execute `cargo clippy --all-targets -- -D warnings` after every code change to guarantee zero warnings.

5. **Black Box Quality Control (QC)**:
   - Test CLI help output (`cv-writer --help`, `cv-writer compile --help`).
   - Test JSON Schema retrieval (`cv-writer schema`).
   - Test valid resume rendering (`cv-writer compile -p profile.json -o resume.pdf`).
   - Test character escaping (`&`, `%`, `$`, `#`, `_`, `{`, `}`, `~`, `^`, `\`).
   - Test error exit codes: missing file (code 1), invalid schema (code 2), LaTeX failure (code 3).
   - Verify zero artifact footprint left behind after tool execution.
