use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;
use tempfile::TempDir;
use thiserror::Error;
use tokio::fs;
use tokio::process::Command;
use tokio::time::timeout;

#[derive(Error, Debug)]
pub enum CompilerError {
    #[error("lualatex executable not found in PATH")]
    LuaLatexNotFound,
    #[error("Compilation timed out after {0:?}")]
    Timeout(Duration),
    #[error("I/O error during compilation: {0}")]
    Io(#[from] std::io::Error),
    #[error("Compilation failed with exit code {exit_code:?}.\nStdout:\n{stdout}\nStderr:\n{stderr}")]
    ExecutionFailed {
        exit_code: Option<i32>,
        stdout: String,
        stderr: String,
    },
    #[error("Generated PDF was not produced or is empty")]
    PdfMissing,
}

#[derive(Debug, Clone)]
pub struct CompileResult {
    /// PDF binary bytes
    pub pdf_bytes: Vec<u8>,
    /// Stdout and Stderr diagnostics from lualatex
    pub log: String,
}

pub struct LatexCompiler {
    binary_path: PathBuf,
    timeout_duration: Duration,
}

impl LatexCompiler {
    pub fn new() -> Result<Self, CompilerError> {
        let binary_path = which::which("lualatex").map_err(|_| CompilerError::LuaLatexNotFound)?;
        Ok(Self {
            binary_path,
            timeout_duration: Duration::from_secs(60),
        })
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout_duration = timeout;
        self
    }

    /// Stateless compile: creates a private tempdir, writes document.tex, runs lualatex twice
    /// (to resolve cross-references and page counts), reads document.pdf into memory,
    /// and drops the TempDir to guarantee zero disk residue.
    pub async fn compile(&self, latex_source: &str) -> Result<CompileResult, CompilerError> {
        // Create an isolated temporary directory (stateless)
        let temp_dir = TempDir::new()?;
        let temp_path = temp_dir.path();
        let tex_file_path = temp_path.join("document.tex");
        let pdf_file_path = temp_path.join("document.pdf");

        fs::write(&tex_file_path, latex_source).await?;

        // Run lualatex pass 1
        let output1 = self.run_lualatex_pass(temp_path, "document.tex").await?;

        // Run lualatex pass 2 for LastPage and TOC cross-referencing
        let output2 = self.run_lualatex_pass(temp_path, "document.tex").await?;

        let combined_log = format!(
            "--- PASS 1 LOG ---\n{}\n--- PASS 2 LOG ---\n{}",
            output1, output2
        );

        if !pdf_file_path.exists() {
            return Err(CompilerError::PdfMissing);
        }

        let pdf_bytes = fs::read(&pdf_file_path).await?;
        if pdf_bytes.is_empty() {
            return Err(CompilerError::PdfMissing);
        }

        // temp_dir is dropped here, automatically unlinking all temp files from disk
        Ok(CompileResult {
            pdf_bytes,
            log: combined_log,
        })
    }

    async fn run_lualatex_pass(&self, dir: &Path, tex_filename: &str) -> Result<String, CompilerError> {
        let mut cmd = Command::new(&self.binary_path);
        cmd.current_dir(dir)
            .arg("-interaction=nonstopmode")
            .arg("-halt-on-error")
            .arg(tex_filename)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let future = async {
            let child = cmd.spawn()?;
            let output = child.wait_with_output().await?;
            Ok::<_, CompilerError>(output)
        };

        let output = match timeout(self.timeout_duration, future).await {
            Ok(res) => res?,
            Err(_) => return Err(CompilerError::Timeout(self.timeout_duration)),
        };

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if !output.status.success() {
            return Err(CompilerError::ExecutionFailed {
                exit_code: output.status.code(),
                stdout,
                stderr,
            });
        }

        Ok(stdout)
    }
}
