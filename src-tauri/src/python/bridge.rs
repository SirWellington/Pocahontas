use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};
use std::time::Duration;

use anyhow::{Context, Result};

/// Configuration for the Python upscaling bridge.
#[derive(Debug, Clone)]
pub struct PythonBridgeConfig {
    /// Path to the Python executable.
    pub python_exe: String,
    /// Path to the upscaling script (e.g., `scripts/upscale.py`).
    pub script_path: String,
    /// Timeout for each upscaling job (seconds).
    pub timeout_seconds: u64,
}

impl Default for PythonBridgeConfig {
    fn default() -> Self {
        Self {
            python_exe: "python".to_string(),
            script_path: "scripts/upscale.py".to_string(),
            timeout_seconds: 300,
        }
    }
}

/// Represents the result of an upscaling job.
#[derive(Debug, Clone)]
pub struct UpscaleResult {
    /// Path to the upscaled output image.
    pub output_path: String,
    /// Exit code from the Python process.
    pub exit_code: i32,
    /// Any error message captured from stderr.
    pub error: Option<String>,
}

/// Bridge between Rust and Python for AI upscaling tasks.
///
/// Architecture:
/// - Rust spawns a Python subprocess per upscaling job.
/// - The Python script receives input/output paths as arguments.
/// - Progress and results are communicated via JSON on stdout.
/// - Errors are captured from stderr.
pub struct PythonBridge {
    config: PythonBridgeConfig,
}

impl PythonBridge {
    pub fn new(config: PythonBridgeConfig) -> Self {
        Self { config }
    }

    pub fn with_defaults() -> Self {
        Self {
            config: PythonBridgeConfig::default(),
        }
    }

    /// Upscales a single image using the Python script.
    ///
    /// The Python script is expected to:
    /// 1. Accept `--input <path>` and `--output <path>` arguments.
    /// 2. Write JSON progress updates to stdout: `{"progress": 0.5}`
    /// 3. Write a final result line: `{"done": true, "output": "<path>"}`
    /// 4. Write errors to stderr.
    pub async fn upscale_image(
        &self,
        input_path: &str,
        output_dir: &str,
        scale_factor: u32,
    ) -> Result<UpscaleResult> {
        let input = Path::new(input_path);
        let output_path = self.generate_output_path(input, output_dir, scale_factor)?;

        // Build the command
        let mut cmd = Command::new(&self.config.python_exe);
        cmd.arg(&self.config.script_path)
            .arg("--input")
            .arg(input_path)
            .arg("--output")
            .arg(&output_path)
            .arg("--scale")
            .arg(scale_factor.to_string())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = cmd.spawn().context("Failed to spawn Python process")?;

        // Read stdout for progress JSON
        let stdout = child.stdout.take().context("Failed to capture stdout")?;
        let stderr = child.stderr.take().context("Failed to capture stderr")?;

        let stdout_reader = BufReader::new(stdout);
        let stderr_reader = BufReader::new(stderr);

        // Process stdout line by line for progress updates
        let progress_handle = tokio::task::spawn_blocking(move || {
            for line in stdout_reader.lines() {
                if let Ok(line) = line {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&line) {
                        if let Some(progress) = json.get("progress").and_then(|v| v.as_f64()) {
                            log::info!("Upscale progress: {:.0}%", progress * 100.0);
                        }
                        if let Some(done) = json.get("done").and_then(|v| v.as_bool()) {
                            if done {
                                log::info!("Upscale complete");
                            }
                        }
                    }
                }
            }
        });

        // Wait for process to complete with timeout
        let exit_status = tokio::time::timeout(
            Duration::from_secs(self.config.timeout_seconds),
            tokio::task::spawn_blocking(move || child.wait()),
        )
        .await
        .context("Upscaling timed out")?
        .context("Python process failed")?;

        // Collect stderr for error messages
        let error_msg = tokio::task::spawn_blocking(move || {
            let mut errors = Vec::new();
            for line in stderr_reader.lines().flatten() {
                errors.push(line);
            }
            errors.join("\n")
        })
        .await
        .unwrap_or_default();

        let error = if error_msg.is_empty() {
            None
        } else {
            Some(error_msg)
        };

        progress_handle.await.ok();

        let exit_status = exit_status?;

        if exit_status.success() {
            Ok(UpscaleResult {
                output_path,
                exit_code: exit_status.code().unwrap_or(0),
                error: None,
            })
        } else {
            Ok(UpscaleResult {
                output_path,
                exit_code: exit_status.code().unwrap_or(-1),
                error,
            })
        }
    }

    /// Upscales multiple images, returning results for each.
    pub async fn upscale_batch(
        &self,
        input_paths: &[String],
        output_dir: &str,
        scale_factor: u32,
    ) -> Vec<(String, Result<UpscaleResult>)> {
        let mut results = Vec::new();

        for path in input_paths {
            let result = self.upscale_image(path, output_dir, scale_factor).await;
            results.push((path.clone(), result));
        }

        results
    }

    /// Generates the output path for an upscaled image.
    fn generate_output_path(
        &self,
        input: &Path,
        output_dir: &str,
        scale_factor: u32,
    ) -> Result<String> {
        let stem = input
            .file_stem()
            .context("Input path has no file stem")?
            .to_string_lossy()
            .to_string();

        let ext = input
            .extension()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let filename = format!("{}_{}x.{}", stem, scale_factor, ext);
        let full_path = PathBuf::from(output_dir).join(filename);

        Ok(full_path.to_string_lossy().to_string())
    }

    /// Checks if the Python environment is available and the script exists.
    pub fn validate(&self) -> Result<()> {
        let output = Command::new(&self.config.python_exe)
            .arg("--version")
            .output()
            .context("Python executable not found. Is Python installed and in PATH?")?;

        if !output.status.success() {
            anyhow::bail!("Python is not available");
        }

        if !Path::new(&self.config.script_path).exists() {
            anyhow::bail!("Upscaling script not found at: {}", self.config.script_path);
        }

        Ok(())
    }
}
