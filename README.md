# CV Writer MCP Server

A stateless [Model Context Protocol (MCP)](https://modelcontextprotocol.io/) server built in **Rust** that allows any MCP-capable AI agent (Claude Desktop, Cursor, Antigravity IDE, custom LLM agents) to synthesize user profiles into professional, publication-ready CVs/resumes formatted with the modernized **Star Rover** template and compiled natively using **LuaLaTeX**.

---

## Architecture & Features

- **Stateless & Ephemeral**: Zero external storage dependencies (no Redis, Postgres, or filesystem retention). Each compilation runs in a private temporary directory and cleans up immediately after producing the in-memory or exported PDF.
- **Modernized LuaLaTeX Template**: Fully migrated from legacy pdfTeX packages to modern LuaLaTeX standards (`fontspec`, OpenType `Fira Sans`, `fontawesome5`, native Unicode handling).
- **Single Invoke Execution**: Runnable via cargo or a single `docker run -i --rm` invocation communicating over standard I/O (JSON-RPC stdio).
- **Self-Descriptive MCP Interface**: Exposes typed schema tools (`get_cv_schema`, `get_sample_profile`, `get_template_info`, `render_cv`) so AI agents understand the resume structure directly.
- **Robust LaTeX Escaping**: Automatically sanitizes special characters (`&`, `%`, `$`, `#`, `_`, `{`, `}`, `~`, `^`, `\`) so user content never crashes the TeX compiler.

---

## Exposed MCP Tools

1. **`get_cv_schema`**:
   - Returns the full JSON Schema for the `CvProfile` structure required by the Star Rover template.
2. **`get_sample_profile`**:
   - Returns a sample valid profile payload demonstrating all supported sections.
3. **`get_template_info`**:
   - Details template metadata, styling palette (`#141E61` navy accent), typography, and formatting rules.
4. **`render_cv`**:
   - **Arguments**:
     - `profile` (Object, required): The structured CV profile matching the schema.
     - `output_path` (String, optional): Destination file path to save the generated PDF.
   - **Output**: Returns status, base64-encoded PDF binary, and byte size.

---

## Running Locally

### Prerequisites
- Rust (1.80+)
- LuaLaTeX (TeXLive with `luatex`, `fontspec`, `firasans`, `fontawesome5`)

### Build & Run
```bash
# Run tests
cargo test

# Run MCP server on stdio
cargo run --release
```

---

## Running with Docker (Single Invoke)

Build the container image:
```bash
docker build -t cv-writer-mcp .
```

Run as a stateless MCP server:
```bash
docker run -i --rm cv-writer-mcp
```

### Configure with Claude Desktop or Antigravity MCP Config

Add to your `mcp_config.json`:
```json
{
  "mcpServers": {
    "cv-writer": {
      "command": "docker",
      "args": ["run", "-i", "--rm", "cv-writer-mcp"]
    }
  }
}
```

Or using native binary:
```json
{
  "mcpServers": {
    "cv-writer": {
      "command": "/path/to/cv-writer/target/release/cv-writer-mcp"
    }
  }
}
```

---

## Scrum Team Roles & Governance

This project was built and is maintained by a 7-role Scrum Team:
- **Product Owner (PO)**: User value, LLM usability, and clean abstraction.
- **Business Analyst (BA)**: Domain entity breakdown, typed schemas, and edge case mappings.
- **Project Manager (PM)**: Constraint enforcement, statelessness, and single-invoke Docker readiness.
- **System Architect (SA)**: Pipeline isolation, MCP stdio protocol compliance, and security.
- **Tech Lead**: LuaLaTeX modernization, OpenType typography, and idiomatic Rust.
- **Developer**: Core implementation, templating engine, and compilation runner.
- **Quality Control (QC)**: Black box verification, input stress testing, and zero disk artifact verification.

Detailed specifications can be found in:
- [.agents/rules/scrum_team.md](file:///home/ahndunn/dev/cv-writer/.agents/rules/scrum_team.md)
- [.agents/skills/cv-writer-scrum-team/SKILL.md](file:///home/ahndunn/dev/cv-writer/.agents/skills/cv-writer-scrum-team/SKILL.md)
