use std::io;
use std::process::Command;

use crate::state::{ApplyPlan, PREFEITURA_USER, SystemSnapshot};

#[derive(Debug, Clone)]
pub struct ApplyOutcome {
    pub reboot_required: bool,
    pub message: String,
}

pub trait WindowsOps {
    fn snapshot(&self) -> io::Result<SystemSnapshot>;
    fn apply(&self, plan: &ApplyPlan) -> io::Result<ApplyOutcome>;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct RealWindowsOps;

impl RealWindowsOps {
    pub fn new() -> Self {
        Self
    }
}

impl WindowsOps for RealWindowsOps {
    fn snapshot(&self) -> io::Result<SystemSnapshot> {
        Ok(SystemSnapshot {
            hostname: read_hostname(),
            domain: read_domain().unwrap_or_else(|_| String::from("WORKGROUP")),
            elevated: is_elevated().unwrap_or(false),
        })
    }

    fn apply(&self, plan: &ApplyPlan) -> io::Result<ApplyOutcome> {
        let mut applied = Vec::new();
        let mut reboot_required = false;

        if let Some(hostname) = plan.hostname.as_deref() {
            rename_computer(hostname)?;
            applied.push(format!("hostname -> {hostname}"));
            reboot_required = true;
        }

        if let Some(password) = plan.password.as_deref() {
            set_password(PREFEITURA_USER, password)?;
            applied.push(format!("password updated for {PREFEITURA_USER}"));
        }

        if let Some(domain) = plan.domain.as_deref() {
            join_domain(domain)?;
            applied.push(format!("domain -> {domain}"));
            reboot_required = true;
        }

        if let Some(username) = plan.create_user.as_deref() {
            create_local_user(username)?;
            applied.push(format!("user {username} created"));
        }

        if applied.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "no actions were selected",
            ));
        }

        let message = if reboot_required {
            format!("Applied: {}. Reboot required.", applied.join(", "))
        } else {
            format!("Applied: {}.", applied.join(", "))
        };

        Ok(ApplyOutcome {
            reboot_required,
            message,
        })
    }
}

fn is_elevated() -> io::Result<bool> {
    let status = Command::new("net").arg("session").status()?;
    Ok(status.success())
}

fn read_hostname() -> String {
    std::env::var("COMPUTERNAME").unwrap_or_else(|_| {
        run_command_output(Command::new("hostname")).unwrap_or_else(|_| String::from("Unknown"))
    })
}

fn read_domain() -> io::Result<String> {
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            "(Get-CimInstance -ClassName Win32_ComputerSystem).Domain",
        ])
        .output()?;

    if !output.status.success() {
        return Err(io::Error::other(format_command_error(
            "powershell",
            &output,
        )));
    }

    let domain = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(if domain.is_empty() {
        String::from("WORKGROUP")
    } else {
        domain
    })
}

fn rename_computer(hostname: &str) -> io::Result<()> {
    let script = format!(
        "Rename-Computer -NewName {} -Force -ErrorAction Stop",
        ps_quote(hostname)
    );
    run_powershell(&script)
}

fn join_domain(domain: &str) -> io::Result<()> {
    let script = format!(
        "Add-Computer -DomainName {} -Force -ErrorAction Stop",
        ps_quote(domain)
    );
    run_powershell(&script)
}

fn create_local_user(username: &str) -> io::Result<()> {
    let output = Command::new("net")
        .args(["user", username, "/add"])
        .output()?;

    if output.status.success() {
        Ok(())
    } else {
        Err(io::Error::other(format_command_error("net user", &output)))
    }
}

fn set_password(user: &str, password: &str) -> io::Result<()> {
    let output = Command::new("net")
        .args(["user", user, password])
        .output()?;

    if output.status.success() {
        Ok(())
    } else {
        Err(io::Error::other(format_command_error("net user", &output)))
    }
}

fn run_powershell(script: &str) -> io::Result<()> {
    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", script])
        .output()?;

    if output.status.success() {
        Ok(())
    } else {
        Err(io::Error::other(format_command_error(
            "powershell",
            &output,
        )))
    }
}

fn run_command_output(mut command: Command) -> io::Result<String> {
    let output = command.output()?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(io::Error::other(format_command_error("command", &output)))
    }
}

fn format_command_error(command: &str, output: &std::process::Output) -> String {
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if stderr.is_empty() && stdout.is_empty() {
        format!("{command} failed")
    } else if stderr.is_empty() {
        format!("{command} failed: {stdout}")
    } else if stdout.is_empty() {
        format!("{command} failed: {stderr}")
    } else {
        format!("{command} failed: {stderr} {stdout}")
    }
}

fn ps_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}
