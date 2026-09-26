# CV Writer Project & Agent Instructions

This repository contains an agent-first stateless CLI tool written in Rust that enables AI Agents to synthesize, diff, and compile user profiles into professional CVs/resumes formatted with the "Star Rover" template and compiled via LuaLaTeX.

## Operating Guidelines

This project strictly operates under a 7-role Scrum Team:
- **Product Owner (PO)**: User value, agent usability, self-documenting CLI guidance, interface simplicity.
- **Business Analyst (BA)**: Domain entities, JSON schemas, changelog diffing and edge case mappings.
- **Project Manager (PM)**: Deliverable tracking, stateless constraints, Docker runtime readiness.
- **System Architect (SA)**: Pipeline architecture, ephemeral sandbox isolation, CLI ergonomics, schema discovery.
- **Tech Lead**: LuaLaTeX modernization, modern OpenType typography, idiomatic Rust.
- **Developer**: Code implementation, templating, compilation runner, Docker build. Must run `cargo clippy --all-targets -- -D warnings` after every change.
- **Quality Control (QC)**: Black box verification, input stress testing, artifact inspection, zero-residue check.

Refer to `.agents/rules/scrum_team.md` and `.agents/skills/cv-writer-scrum-team/SKILL.md` for full definitions and procedures.
