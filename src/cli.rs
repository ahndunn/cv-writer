use clap::{Args, Parser, Subcommand};
use schemars::schema_for;
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use crate::changelog::record_changelog;
use crate::compiler::LatexCompiler;
use crate::schema::{CvProfile, sample_cv_profile};
use crate::template::{STAR_ROVER_TEMPLATE, render_latex};

/// Agent-First Stateless CLI for synthesizing, diffing, and compiling CVs into PDF using LuaLaTeX.
///
/// Designed specifically for LLMs and autonomous AI agents to build publication-grade
/// resumes without dealing with LaTeX syntax, escaping quirks, or persistent state.
#[derive(Parser, Debug)]
#[command(
    name = "cv-writer",
    version,
    about = "Agent-first stateless CLI tool for synthesizing, diffing, and compiling CVs into PDF using LuaLaTeX.",
    after_help = r#"LLM AGENT USAGE GUIDE:
1. Inspect the JSON Schema:
   Run `cv-writer schema` to understand the expected profile JSON fields.

2. Get a complete reference example:
   Run `cv-writer sample` to inspect a realistic, valid JSON profile.

3. Compile a profile into a PDF:
   Run `cv-writer compile -p ./profile.json -o ./output/resume.pdf`
   Pass `--changelog-dir ./changelogs/` to automatically track and audit tailoring history.

4. Pipe profile directly via STDIN:
   `cat profile.json | cv-writer compile -p - -o ./resume.pdf`

5. Inspect generated TeX without compilation:
   `cv-writer compile -p ./profile.json -o ./resume.pdf --emit-tex`

EXIT CODES:
  0: Success (PDF generated atomically).
  1: I/O or input file error (e.g. missing profile file).
  2: Schema validation error (invalid JSON structure against CvProfile schema).
  3: LaTeX compilation error (LuaLaTeX syntax error or engine failure).
"#
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Compile a structured JSON profile into a publication-ready PDF using LuaLaTeX.
    #[command(after_help = r#"EXAMPLES:
  # Basic compilation from a file:
  cv-writer compile --profile ./user_profile.json --output ./dist/cv.pdf

  # Read profile directly from standard input:
  cat profile.json | cv-writer compile --profile - --output ./dist/cv.pdf

  # Compile and automatically save audit changelogs (diffs):
  cv-writer compile -p ./profile.json -o ./cv.pdf --changelog-dir ./changelogs/

  # Generate both the PDF and keep the raw generated .tex file for verification:
  cv-writer compile -p ./profile.json -o ./cv.pdf --emit-tex
"#)]
    Compile(CompileArgs),

    /// Output the canonical JSON Schema for the CvProfile model.
    ///
    /// AI agents should invoke this to validate or build structured resume JSON.
    #[command(after_help = r#"EXAMPLES:
  cv-writer schema
  cv-writer schema > cv_schema.json
"#)]
    Schema,

    /// Output a complete, realistic sample profile conforming to the schema.
    ///
    /// Ideal for few-shot prompting, structural reference, and template testing.
    #[command(after_help = r#"EXAMPLES:
  cv-writer sample
  cv-writer sample > sample_profile.json
"#)]
    Sample,

    /// Display detailed template and typography metadata.
    #[command(after_help = r#"EXAMPLES:
  cv-writer template-info
"#)]
    TemplateInfo,
}

#[derive(Args, Debug)]
pub struct CompileArgs {
    /// Path to the input profile JSON file, or '-' to read from STDIN.
    #[arg(short, long, value_name = "PATH")]
    pub profile: PathBuf,

    /// Destination file path where the generated PDF will be written.
    #[arg(short, long, value_name = "FILE")]
    pub output: PathBuf,

    /// Optional directory to record timestamped profile snapshots and markdown changelogs.
    ///
    /// When specified, every run computes a diff against previous profiles in this directory,
    /// tracking added/removed skills, revised role bullet points, and section updates.
    #[arg(short, long, value_name = "DIR")]
    pub changelog_dir: Option<PathBuf>,

    /// Also emit the rendered LaTeX source next to the output PDF (e.g. `<output>.tex`).
    #[arg(long)]
    pub emit_tex: bool,

    /// Validate the profile and render the template without running the LuaLaTeX compiler.
    #[arg(long)]
    pub dry_run: bool,
}

impl Cli {
    pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
        let cli = Cli::parse();
        cli.execute().await
    }

    pub async fn execute(&self) -> Result<(), Box<dyn std::error::Error>> {
        match &self.command {
            Commands::Schema => {
                let schema = schema_for!(CvProfile);
                let json_schema = serde_json::to_string_pretty(&schema)?;
                println!("{}", json_schema);
                Ok(())
            }
            Commands::Sample => {
                let sample = sample_cv_profile();
                let sample_json = serde_json::to_string_pretty(&sample)?;
                println!("{}", sample_json);
                Ok(())
            }
            Commands::TemplateInfo => {
                let info = serde_json::json!({
                    "template": "Star Rover (Native LuaLaTeX with fontspec & OpenType)",
                    "engine": "LuaLaTeX",
                    "typography": {
                        "primary_font": "Fira Sans",
                        "icons": "FontAwesome 5",
                        "accent_color": "#141E61 (Navy/Indigo)",
                        "secondary_color": "gray"
                    },
                    "sections_supported": [
                        "contact (name, phone, email, github, linkedin, website, location)",
                        "summary",
                        "education (institution, degree, dates, highlights)",
                        "experience (company, location, roles: [title, dates, highlights])",
                        "projects (name, url, dates, highlights)",
                        "skills (category, items)",
                        "certifications (name, issuer, date)",
                        "publications (citation, url)",
                        "awards (title, date, summary)"
                    ],
                    "template_size_bytes": STAR_ROVER_TEMPLATE.len()
                });
                println!("{}", serde_json::to_string_pretty(&info)?);
                Ok(())
            }
            Commands::Compile(args) => run_compile(args).await,
        }
    }
}

async fn run_compile(args: &CompileArgs) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Read profile JSON from file or stdin
    let raw_profile_json = if args.profile.as_os_str() == "-" {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer).map_err(|e| {
            eprintln!("Error reading profile from STDIN: {}", e);
            std::process::exit(1);
        })?;
        buffer
    } else {
        fs::read_to_string(&args.profile).map_err(|e| {
            eprintln!(
                "Error opening profile file '{}': {}\nAgent hint: Check that the file exists and is accessible.",
                args.profile.display(),
                e
            );
            std::process::exit(1);
        })?
    };

    // 2. Deserialize and validate profile against CvProfile schema
    let profile: CvProfile = match serde_json::from_str(&raw_profile_json) {
        Ok(p) => p,
        Err(e) => {
            eprintln!(
                "Schema validation error in profile JSON:\n  {}\n\nAgent hint: Call `cv-writer schema` or `cv-writer sample` to inspect the canonical structure.",
                e
            );
            std::process::exit(2);
        }
    };

    // 3. Changelog recording if requested
    if let Some(ref changelog_dir) = args.changelog_dir {
        match record_changelog(changelog_dir, &profile) {
            Ok(report) => {
                eprintln!(
                    "Changelog recorded in '{}'. Baseline: {}.",
                    changelog_dir.display(),
                    report
                        .previous_timestamp
                        .as_deref()
                        .unwrap_or("Initial snapshot")
                );
            }
            Err(e) => {
                eprintln!("Warning: Failed to record changelog: {}", e);
            }
        }
    }

    // 4. Render Tera LaTeX template
    let rendered_tex = match render_latex(&profile) {
        Ok(tex) => tex,
        Err(e) => {
            eprintln!("Template rendering error: {}", e);
            std::process::exit(3);
        }
    };

    // Emit .tex file if requested
    if args.emit_tex {
        let tex_path = args.output.with_extension("tex");
        if let Some(parent) = tex_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Err(e) = fs::write(&tex_path, &rendered_tex) {
            eprintln!(
                "Warning: Failed to write .tex file to '{}': {}",
                tex_path.display(),
                e
            );
        } else {
            eprintln!("Emitted LaTeX source to '{}'", tex_path.display());
        }
    }

    if args.dry_run {
        println!("Dry run complete. Profile validated and LaTeX template rendered successfully.");
        return Ok(());
    }

    // 5. Compile with LuaLaTeX
    let compiler = match LatexCompiler::new() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Compiler error: {}", e);
            eprintln!(
                "Agent hint: Ensure `luatex` and TeXLive are installed on the system/container."
            );
            std::process::exit(3);
        }
    };

    let compile_res = match compiler.compile(&rendered_tex).await {
        Ok(res) => res,
        Err(e) => {
            eprintln!("LuaLaTeX compilation failed:\n{}", e);
            std::process::exit(3);
        }
    };

    // 6. Write output PDF atomically
    write_atomic(&args.output, &compile_res.pdf_bytes)?;

    println!(
        "Successfully compiled CV for '{}' to '{}' ({} bytes).",
        profile.contact.name,
        args.output.display(),
        compile_res.pdf_bytes.len()
    );

    Ok(())
}

fn write_atomic(dest: &Path, content: &[u8]) -> io::Result<()> {
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }
    // Write to a temporary file next to destination and rename to guarantee atomic replace
    let temp_path = dest.with_extension("tmp.pdf");
    fs::write(&temp_path, content)?;
    fs::rename(temp_path, dest)?;
    Ok(())
}
