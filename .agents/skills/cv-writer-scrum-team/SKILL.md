---
name: cv-writer-scrum-team
description: >-
  Orchestrates the Scrum Team workflow (PO, BA, PM, SA, Tech Lead, Developer, QC)
  for building and maintaining the stateless Rust MCP LuaLaTeX CV Writer server.
---

# CV Writer Scrum Team Skill

This skill defines the operational workflow, role checklists, and review gates for the CV Writer MCP project.

## Operational Lifecycle

1. **Sprint Planning & Vision (PO + PM)**:
   - Establish sprint goals.
   - Guard statelessness, single-invoke Docker portability, and zero-state constraints.

2. **Analysis & Schema Definition (BA + PO)**:
   - Define canonical resume schema (Contact, Education, Experience, Skills, Certifications, Publications, Awards, Projects).
   - Ensure AI Agent friendliness with self-descriptive field schemas.

3. **Architectural & Modernization Review (SA + Tech Lead)**:
   - Template migration to modern LuaLaTeX standards (OpenType fonts via `fontspec`, utf-8 native handling).
   - Rust architecture: MCP stdio server handling JSON-RPC requests, template renderer, isolated ephemeral compilation in `/tmp` ramfs with automatic sweep.

4. **Implementation (Developer)**:
   - Modernized `.tex` template with Tera/Askama tokens.
   - Rust crates: `serde`, `serde_json`, `tokio`, `tera`, `tempfile`, base64 encoding.
   - MCP Tools:
     - `get_cv_schema`: Returns the full JSON schema for an AI Agent to inspect.
     - `render_cv`: Accepts structured profile data, validates, compiles via LuaLaTeX, returns base64 PDF and compilation diagnostics.
     - `get_sample_profile`: Returns example valid input data matching Star Rover layout.
   - Production Dockerfile with TeXLive LuaLaTeX + Rust binary.

5. **Black Box Quality Control (QC)**:
   - Test MCP tool discovery (`tools/list`).
   - Test JSON Schema retrieval (`tools/call` `get_cv_schema`).
   - Test valid resume rendering (`tools/call` `render_cv`).
   - Test character escaping (`&`, `%`, `$`, `#`, `_`, `{`, `}`, `~`, `^`, `\`).
   - Test error handling on malformed syntax or empty payloads.
   - Verify zero artifact footprint left behind after tool execution.
