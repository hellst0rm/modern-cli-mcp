// modern-cli-mcp/src/tools/executor.rs
use std::process::Stdio;
use tokio::process::Command;

#[derive(Debug, Clone, Default)]
pub struct CommandExecutor;

impl CommandExecutor {
    pub fn new() -> Self {
        Self
    }

    pub async fn run(&self, cmd: &str, args: &[&str]) -> Result<CommandOutput, String> {
        self.run_in_dir(cmd, args, None).await
    }

    pub async fn run_in_dir(
        &self,
        cmd: &str,
        args: &[&str],
        working_dir: Option<&str>,
    ) -> Result<CommandOutput, String> {
        let cmd_path =
            which::which(cmd).map_err(|_| format!("Command '{}' not found in PATH", cmd))?;

        let mut command = Command::new(&cmd_path);
        command
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        if let Some(dir) = working_dir {
            command.current_dir(dir);
        }

        let output = command
            .output()
            .await
            .map_err(|e| format!("Failed to execute {}: {}", cmd, e))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        Ok(CommandOutput {
            success: output.status.success(),
            exit_code: output.status.code(),
            stdout,
            stderr,
        })
    }

    pub async fn run_with_stdin(
        &self,
        cmd: &str,
        args: &[&str],
        stdin_data: &str,
    ) -> Result<CommandOutput, String> {
        use tokio::io::AsyncWriteExt;

        let cmd_path =
            which::which(cmd).map_err(|_| format!("Command '{}' not found in PATH", cmd))?;

        let mut child = Command::new(&cmd_path)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn {}: {}", cmd, e))?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(stdin_data.as_bytes())
                .await
                .map_err(|e| format!("Failed to write stdin: {}", e))?;
        }

        let output = child
            .wait_with_output()
            .await
            .map_err(|e| format!("Failed to wait for {}: {}", cmd, e))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        Ok(CommandOutput {
            success: output.status.success(),
            exit_code: output.status.code(),
            stdout,
            stderr,
        })
    }
}

#[derive(Debug)]
pub struct CommandOutput {
    pub success: bool,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

impl CommandOutput {
    pub fn to_result_string(&self) -> String {
        if self.success {
            if self.stdout.is_empty() {
                "(no output)".to_string()
            } else {
                self.stdout.clone()
            }
        } else {
            let mut result = String::new();
            if !self.stderr.is_empty() {
                result.push_str(&self.stderr);
            }
            if !self.stdout.is_empty() {
                if !result.is_empty() {
                    result.push_str("\n");
                }
                result.push_str(&self.stdout);
            }
            if result.is_empty() {
                format!(
                    "Command failed with exit code: {}",
                    self.exit_code.unwrap_or(-1)
                )
            } else {
                result
            }
        }
    }
}
