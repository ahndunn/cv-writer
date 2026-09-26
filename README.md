# CV Writer

An **Agent-First, Stateless CLI Tool** written in **Rust** that empowers AI Agents and engineers to synthesize, diff, and compile user profiles into publication-ready CVs/resumes formatted with the modern **Star Rover** template and compiled natively using **LuaLaTeX**.

---

## ⚡ Agent-First Architecture & Design

- **Agent-First CLI (`clap`)**: Specifically engineered for LLMs and autonomous agents. Running `cv-writer --help` or `cv-writer compile --help` outputs comprehensive workflow instructions, inline schema cues, and deterministic exit code guidance.
- **Stateless & Ephemeral**: Zero external storage or database dependencies. Each compilation executes inside a sandboxed temporary directory and leaves zero disk residue.
- **Automated Changelogs & Profile Diffs**: Providing `--changelog-dir <DIR>` automatically compares the current CV against previous iterations, persisting timestamped JSON baselines and clean Markdown diff reports (tracking added/removed skills, revised role bullet points, and section updates).
- **Direct Subcommands for Schema Discovery**: Agents can run `cv-writer schema` or `cv-writer sample` directly via bash tools to validate profiles without needing external reference files.
- **Modernized LuaLaTeX Engine**: Uses `fontspec`, OpenType `Fira Sans`, `fontawesome5`, and native UTF-8 handling.
- **Automatic LaTeX Sanitization**: Special characters (`&`, `%`, `$`, `#`, `_`, `{`, `}`, `~`, `^`, `\`) are escaped automatically by the Tera templating layer so user text never crashes the TeX compiler.

---

## 🚀 CLI Commands & Subcommands

### 1. `cv-writer compile`
Synthesize and compile a profile into a PDF document:
```bash
# Basic compilation
cv-writer compile --profile ./profile.json --output ./build/resume.pdf

# With automated changelog diff tracking
cv-writer compile -p ./profile.json -o ./build/resume.pdf --changelog-dir ./changelogs/

# Pipe directly from STDIN
cat profile.json | cv-writer compile -p - -o ./build/resume.pdf

# Emit raw LaTeX source for debugging or manual inspection
cv-writer compile -p ./profile.json -o ./build/resume.pdf --emit-tex

# Dry-run validation (verifies schema and renders template without invoking LuaLaTeX)
cv-writer compile -p ./profile.json -o ./build/resume.pdf --dry-run
```

### 2. `cv-writer schema`
Prints the canonical JSON Schema for the `CvProfile` data structure:
```bash
cv-writer schema > cv_schema.json
```

### 3. `cv-writer sample`
Outputs a realistic, valid JSON profile demonstrating all supported sections:
```bash
cv-writer sample > sample_profile.json
```

### 4. `cv-writer template-info`
Outputs typography, palette (`#141E61` navy accent), and template section capabilities in JSON.

---

## 🚦 Exit Code Contract

| Exit Code | Meaning | Remediation Guidance for Agents |
| :---: | :--- | :--- |
| **`0`** | **Success** | PDF generated and placed atomically at `--output`. |
| **`1`** | **I/O Error** | Check that input file exists, path is correct, or permissions are granted. |
| **`2`** | **Schema Validation Error** | Input JSON does not match the `CvProfile` schema. Call `cv-writer schema` or `cv-writer sample`. |
| **`3`** | **Compilation / TeX Error** | LaTeX syntax error or missing LuaLaTeX environment in runtime. |

---

## 🐳 Running with Docker (Single Invoke)

Build the container image:
```bash
docker build -t cv-writer .
```

Run compilation mounted to host workspace:
```bash
docker run --rm \
  -v "$(pwd)":/workspace \
  cv-writer compile \
    --profile /workspace/profile.json \
    --output /workspace/cv.pdf \
    --changelog-dir /workspace/changelogs
```

Inspect schema from container:
```bash
docker run --rm cv-writer schema
```

---

## 🛠 Local Development & Testing

### Prerequisites
- Rust (1.80+)
- LuaLaTeX (TeXLive with `luatex`, `fontspec`, `firasans`, `fontawesome5`)

```bash
# Run unit & integration test suites
cargo test

# Run strict Clippy check (zero warnings tolerated)
cargo clippy --all-targets -- -D warnings

# Build optimized binary
cargo build --release
```

---

## 👥 Scrum Team Roles & Governance

This project is governed by a strict 7-role Scrum Team:
- **Product Owner (PO)**: User value, LLM agent usability, self-documenting CLI guidance.
- **Business Analyst (BA)**: Domain entities, JSON schemas, changelog diffing and tailoring audits.
- **Project Manager (PM)**: Delivery orchestration, stateless constraints, Docker runtime readiness.
- **System Architect (SA)**: Pipeline architecture, ephemeral sandbox isolation, CLI ergonomics.
- **Tech Lead**: LuaLaTeX modernization, modern OpenType typography, idiomatic Rust.
- **Developer**: Code implementation, templating, compilation runner, Docker build.
- **Quality Control (QC)**: Black box verification, input stress testing, exit code compliance.

Detailed specifications can be found in:
- [.agents/rules/scrum_team.md](file://.agents/rules/scrum_team.md)
- [.agents/skills/cv-writer-scrum-team/SKILL.md](file://.agents/skills/cv-writer-scrum-team/SKILL.md)
