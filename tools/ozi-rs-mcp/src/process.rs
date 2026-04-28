use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::Instant,
};

use crate::evidence::{EvidenceMetadata, EvidencePaths, EvidenceStatus, started_at_now};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandOutput {
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub exit_code: i32,
}

pub trait EvidenceCommand {
    fn command_line(&self) -> Vec<String>;
    fn run(&self) -> anyhow::Result<CommandOutput>;
}

#[derive(Debug, Clone)]
pub struct FakeCommand {
    command: Vec<String>,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    exit_code: i32,
}

#[derive(Debug, Clone)]
pub struct RealCommand {
    program: String,
    args: Vec<String>,
    current_dir: Option<PathBuf>,
}

impl RealCommand {
    pub fn new(program: impl Into<String>) -> Self {
        Self {
            program: program.into(),
            args: Vec::new(),
            current_dir: None,
        }
    }

    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into());
        self
    }

    pub fn current_dir(mut self, current_dir: impl AsRef<Path>) -> Self {
        self.current_dir = Some(current_dir.as_ref().to_path_buf());
        self
    }
}

impl EvidenceCommand for RealCommand {
    fn command_line(&self) -> Vec<String> {
        let mut command = Vec::with_capacity(self.args.len() + 1);
        command.push(self.program.clone());
        command.extend(self.args.clone());
        command
    }

    fn run(&self) -> anyhow::Result<CommandOutput> {
        let mut command = Command::new(&self.program);
        command.args(&self.args);
        if let Some(current_dir) = &self.current_dir {
            command.current_dir(current_dir);
        }

        let output = command.output()?;
        Ok(CommandOutput {
            stdout: output.stdout,
            stderr: output.stderr,
            exit_code: output.status.code().unwrap_or(-1),
        })
    }
}

impl FakeCommand {
    pub fn new(program: impl Into<String>) -> Self {
        Self {
            command: vec![program.into()],
            stdout: Vec::new(),
            stderr: Vec::new(),
            exit_code: 0,
        }
    }

    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.command.push(arg.into());
        self
    }

    pub fn stdout(mut self, stdout: impl AsRef<[u8]>) -> Self {
        self.stdout = stdout.as_ref().to_vec();
        self
    }

    pub fn stderr(mut self, stderr: impl AsRef<[u8]>) -> Self {
        self.stderr = stderr.as_ref().to_vec();
        self
    }

    pub fn exit_code(mut self, exit_code: i32) -> Self {
        self.exit_code = exit_code;
        self
    }
}

impl EvidenceCommand for FakeCommand {
    fn command_line(&self) -> Vec<String> {
        self.command.clone()
    }

    fn run(&self) -> anyhow::Result<CommandOutput> {
        Ok(CommandOutput {
            stdout: self.stdout.clone(),
            stderr: self.stderr.clone(),
            exit_code: self.exit_code,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapturedCommandResult {
    pub metadata: EvidenceMetadata,
    pub stdout_path: PathBuf,
    pub stderr_path: PathBuf,
}

pub fn run_command_with_evidence(
    tool: &str,
    paths: &EvidencePaths,
    command: &impl EvidenceCommand,
) -> anyhow::Result<CapturedCommandResult> {
    let started_at = started_at_now();
    let started = Instant::now();
    let output = command.run()?;
    let duration_ms = started.elapsed().as_millis();

    let stdout_path = paths.path_for(tool, "stdout.txt")?;
    let stderr_path = paths.path_for(tool, "stderr.txt")?;

    paths.prepare_file(&stdout_path)?;
    paths.prepare_file(&stderr_path)?;
    fs::write(&stdout_path, &output.stdout)?;
    fs::write(&stderr_path, &output.stderr)?;

    let status = if output.exit_code == 0 {
        EvidenceStatus::Passed
    } else {
        EvidenceStatus::Failed
    };
    let error_kind = (output.exit_code != 0).then(|| "exit_code".to_owned());

    let metadata = EvidenceMetadata {
        tool: tool.to_owned(),
        started_at,
        duration_ms,
        command: command.command_line(),
        exit_code: Some(output.exit_code),
        stdout_path: paths.relative_display(&stdout_path)?,
        stderr_path: paths.relative_display(&stderr_path)?,
        artifact_paths: Vec::new(),
        status,
        error_kind,
    };

    Ok(CapturedCommandResult {
        metadata,
        stdout_path,
        stderr_path,
    })
}
