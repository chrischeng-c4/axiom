// HANDWRITE-BEGIN gap="missing-generator:logic:540e8cc0" tracker="pending-tracker" reason="Add the first resident light-shell session boundary. The session captures cwd/env, plans a command string through the existing command planner, runs native command stages in process, and returns a Bash fallback plan for unsupported or unproven command strings."
use std::{
    env,
    ffi::OsString,
    path::{Path, PathBuf},
    process::ExitCode,
};

use anyhow::Result;

use crate::command_planner::{self, CommandPlan, ExternalImplementation, ExternalPlan, NativePlan};

/// @spec projects/cap/tech-design/logic/design-resident-light-shell-with-dynamic-bash-fallback.md#logic
#[derive(Debug, Clone)]
pub struct ResidentLightShellSession {
    cwd: PathBuf,
    env: Vec<(OsString, OsString)>,
}

/// @spec projects/cap/tech-design/logic/design-resident-light-shell-with-dynamic-bash-fallback.md#logic
#[derive(Debug, Clone)]
pub enum ResidentLightShellPlan {
    Native(NativePlan),
    BashFallback(ExternalPlan),
}

/// @spec projects/cap/tech-design/logic/design-resident-light-shell-with-dynamic-bash-fallback.md#logic
#[derive(Debug, Clone)]
pub enum ResidentLightShellRun {
    Native(ExitCode),
    BashFallback(ExternalPlan),
}

/// @spec projects/cap/tech-design/logic/design-resident-light-shell-with-dynamic-bash-fallback.md#logic
impl ResidentLightShellSession {
    pub fn capture() -> Self {
        Self {
            cwd: env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            env: env::vars_os().collect(),
        }
    }

    pub fn cwd(&self) -> &Path {
        &self.cwd
    }

    pub fn env_len(&self) -> usize {
        self.env.len()
    }

    pub fn plan_command_string(
        &self,
        command: &str,
        label: Option<String>,
    ) -> ResidentLightShellPlan {
        match command_planner::plan_shell(command, label.clone()) {
            CommandPlan::Native(native) => ResidentLightShellPlan::Native(native),
            CommandPlan::External(_) => ResidentLightShellPlan::BashFallback(bash_fallback(
                command,
                label.or_else(|| Some(command.trim().to_string())),
            )),
        }
    }

    pub fn run_command_string(
        &self,
        command: &str,
        label: Option<String>,
    ) -> Result<ResidentLightShellRun> {
        match self.plan_command_string(command, label) {
            ResidentLightShellPlan::Native(native) => Ok(ResidentLightShellRun::Native(
                command_planner::run_native(&native)?,
            )),
            ResidentLightShellPlan::BashFallback(plan) => {
                Ok(ResidentLightShellRun::BashFallback(plan))
            }
        }
    }

    #[cfg(test)]
    fn run_command_string_to(
        &self,
        command: &str,
        stdout: &mut dyn std::io::Write,
        stderr: &mut dyn std::io::Write,
    ) -> Result<ResidentLightShellTestRun> {
        match self.plan_command_string(command, None) {
            ResidentLightShellPlan::Native(native) => {
                let code = command_planner::run_native_to(&native, stdout, stderr)?;
                Ok(ResidentLightShellTestRun::Native(code))
            }
            ResidentLightShellPlan::BashFallback(plan) => {
                Ok(ResidentLightShellTestRun::BashFallback(plan))
            }
        }
    }
}

fn bash_fallback(command: &str, label: Option<String>) -> ExternalPlan {
    ExternalPlan {
        program: "bash".to_string(),
        args: vec!["-lc".to_string(), command.to_string()],
        label,
        original: command.to_string(),
        implementation: ExternalImplementation::Original,
        reason: "resident light shell could not prove an in-process native stage; falling back to bash -lc original command".to_string(),
        fallback: None,
    }
}

#[cfg(test)]
#[derive(Debug)]
enum ResidentLightShellTestRun {
    Native(i32),
    BashFallback(ExternalPlan),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, process::Command};

    fn make_large_list_dir(root: &Path) -> PathBuf {
        let dir = root.join("resident-ls-large");
        fs::create_dir(&dir).expect("create large list dir");
        for idx in 0..1024 {
            fs::write(dir.join(format!("entry-{idx:04}.txt")), b"x").expect("write fixture");
        }
        dir
    }

    #[test]
    fn resident_light_shell_captures_session_context() {
        let session = ResidentLightShellSession::capture();
        assert!(session.cwd().exists());
        assert!(session.env_len() > 0);
    }

    #[test]
    fn resident_light_shell_native_ls_matches_original_output() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let dir = make_large_list_dir(temp.path());
        let command = format!("ls -1 {}", dir.display());
        let session = ResidentLightShellSession::capture();

        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let run = session.run_command_string_to(&command, &mut stdout, &mut stderr)?;
        match run {
            ResidentLightShellTestRun::Native(code) => assert_eq!(code, 0),
            ResidentLightShellTestRun::BashFallback(plan) => {
                panic!("expected native resident path, got fallback: {plan:?}")
            }
        }

        let original = Command::new("/bin/ls")
            .args(["-1", &dir.display().to_string()])
            .output()?;
        assert_eq!(Some(0), original.status.code());
        assert_eq!(stdout, original.stdout);
        assert_eq!(stderr, original.stderr);
        Ok(())
    }

    #[test]
    fn resident_light_shell_fallback_preserves_exact_bash_command() -> Result<()> {
        let command = "printf 'one\\ntwo\\n' | wc -l";
        let session = ResidentLightShellSession::capture();
        let plan = match session.plan_command_string(command, None) {
            ResidentLightShellPlan::Native(native) => {
                panic!("expected Bash fallback, got native: {native:?}")
            }
            ResidentLightShellPlan::BashFallback(plan) => plan,
        };

        assert_eq!(plan.program, "bash");
        assert_eq!(plan.args, vec!["-lc".to_string(), command.to_string()]);
        assert_eq!(plan.original, command);

        let fallback = Command::new(&plan.program).args(&plan.args).output()?;
        let original = Command::new("bash").args(["-lc", command]).output()?;
        assert_eq!(fallback.status.code(), original.status.code());
        assert_eq!(fallback.stdout, original.stdout);
        assert_eq!(fallback.stderr, original.stderr);
        Ok(())
    }
}

// HANDWRITE-END
