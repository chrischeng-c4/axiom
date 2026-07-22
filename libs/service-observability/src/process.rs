// HANDWRITE-BEGIN gap="missing-generator:logic:shared-process-resource-sampling" tracker="#1777" reason="Portable, safe process RSS and CPU sampling is shared by service performance and soak evidence."
use std::process::Command;

use anyhow::{bail, Context, Result};

/// A point-in-time process resource sample normalized across macOS and Linux.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProcessUsage {
    pub cpu_seconds: f64,
    pub rss_bytes: u64,
}

/// Sample one process through the portable `ps` surface without unsafe FFI.
pub fn process_usage(pid: u32) -> Result<ProcessUsage> {
    let output = Command::new("ps")
        .args(["-o", "rss=", "-o", "time=", "-p", &pid.to_string()])
        .output()
        .with_context(|| format!("sample process {pid} with ps"))?;
    if !output.status.success() {
        bail!("ps failed for process {pid}: {}", output.status);
    }
    parse_ps_usage(&String::from_utf8(output.stdout).context("ps output is not UTF-8")?)
}

fn parse_ps_usage(output: &str) -> Result<ProcessUsage> {
    let mut fields = output.split_whitespace();
    let rss_kib = fields
        .next()
        .context("ps output is missing RSS")?
        .parse::<u64>()
        .context("ps RSS is not numeric")?;
    let cpu = fields.next().context("ps output is missing CPU time")?;
    Ok(ProcessUsage {
        cpu_seconds: parse_cpu_time(cpu)?,
        rss_bytes: rss_kib.saturating_mul(1024),
    })
}

fn parse_cpu_time(value: &str) -> Result<f64> {
    let (days, clock) = if let Some((days, clock)) = value.split_once('-') {
        (
            days.parse::<f64>()
                .context("ps CPU day count is not numeric")?,
            clock,
        )
    } else {
        (0.0, value)
    };
    let fields = clock
        .split(':')
        .map(str::parse::<f64>)
        .collect::<std::result::Result<Vec<_>, _>>()
        .context("ps CPU clock is not numeric")?;
    let seconds = match fields.as_slice() {
        [minutes, seconds] => minutes * 60.0 + seconds,
        [hours, minutes, seconds] => hours * 3_600.0 + minutes * 60.0 + seconds,
        _ => bail!("unexpected ps CPU time value {value}"),
    };
    Ok(days * 86_400.0 + seconds)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_macos_and_linux_ps_time_shapes() {
        assert_eq!(
            parse_ps_usage("  12345 1:02.50\n").unwrap(),
            ProcessUsage {
                cpu_seconds: 62.5,
                rss_bytes: 12_641_280,
            }
        );
        assert_eq!(
            parse_ps_usage("42 2-03:04:05\n").unwrap(),
            ProcessUsage {
                cpu_seconds: 183_845.0,
                rss_bytes: 43_008,
            }
        );
    }
}
// HANDWRITE-END
