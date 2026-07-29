"""Runnable Typer frontend for the Guard Python reference product.

@spec #2931
"""

from __future__ import annotations

from dataclasses import dataclass
from enum import Enum
import os
from pathlib import Path
import shutil
from typing import Annotated

import typer

from evidence import EvidenceCommand
from policy import PolicyProfile
from report import GuardReport
from scan import ScanOptions, scan_path

__aw_artifact_id__ = "artifact:guard/design-cli"
__aw_work_item__ = "2931"


class ProfileArg(str, Enum):
    BASELINE_STATIC = "baseline-static"
    SECURITY_LINT = "security-lint"
    STRICT = "strict"

    def policy(self) -> PolicyProfile:
        return {
            ProfileArg.BASELINE_STATIC: PolicyProfile.BASELINE_STATIC,
            ProfileArg.SECURITY_LINT: PolicyProfile.SECURITY_LINT,
            ProfileArg.STRICT: PolicyProfile.STRICT,
        }[self]


@dataclass(frozen=True)
class OutputOptions:
    compact: bool = False
    human: bool = False


_OUTPUT = OutputOptions()

app = typer.Typer(
    name="guard",
    help="Security posture gate with JSON on stdout by default.",
    no_args_is_help=True,
)


@app.callback()
def root(
    compact: Annotated[
        bool,
        typer.Option("--compact", help="Emit one dense JSON report line."),
    ] = False,
    human: Annotated[
        bool,
        typer.Option("--human", help="Also render a short summary to stderr."),
    ] = False,
) -> None:
    """Configure the public report projection."""
    global _OUTPUT
    _OUTPUT = OutputOptions(compact, human)


@app.command("scan", help="Run a security profile over a file or directory.")
def scan(
    path: Annotated[
        Path,
        typer.Argument(help="File or directory to scan."),
    ] = Path("."),
    profile: Annotated[
        ProfileArg,
        typer.Option("--profile", help="Security policy profile."),
    ] = ProfileArg.BASELINE_STATIC,
    vat_runner: Annotated[
        list[str],
        typer.Option("--vat-runner", help="Run a named isolated Vat runner."),
    ] = [],
    vat_command: Annotated[
        list[str],
        typer.Option("--vat-command", help="Run one exact Vat evidence command."),
    ] = [],
    rig_dir: Annotated[
        list[Path],
        typer.Option("--rig-dir", help="Run Rig scenarios below a directory."),
    ] = [],
    rig_scenario: Annotated[
        list[Path],
        typer.Option("--rig-scenario", help="Run one Rig scenario file."),
    ] = [],
    rig_command: Annotated[
        list[str],
        typer.Option("--rig-command", help="Run one exact Rig evidence command."),
    ] = [],
    meter_target: Annotated[
        list[Path],
        typer.Option("--meter-target", help="Run Meter against one target."),
    ] = [],
    meter_command: Annotated[
        list[str],
        typer.Option("--meter-command", help="Run one exact Meter command."),
    ] = [],
    arena_spec: Annotated[
        list[Path],
        typer.Option("--arena-spec", help="Run one legacy Arena comparison."),
    ] = [],
    arena_command: Annotated[
        list[str],
        typer.Option("--arena-command", help="Run one legacy Arena command."),
    ] = [],
    no_persist: Annotated[
        bool,
        typer.Option("--no-persist", help="Do not persist the last report."),
    ] = False,
    compact: Annotated[
        bool,
        typer.Option("--compact", help="Emit one dense JSON report line."),
    ] = False,
    human: Annotated[
        bool,
        typer.Option("--human", help="Also render a short summary to stderr."),
    ] = False,
) -> None:
    """Scan source and fold configured dynamic-security evidence."""
    commands = _evidence_commands(
        path,
        vat_runner,
        vat_command,
        rig_dir,
        rig_scenario,
        rig_command,
        meter_target,
        meter_command,
        arena_spec,
        arena_command,
    )
    report = scan_path(
        path,
        ScanOptions(
            profile=profile.policy(),
            evidence_commands=tuple(commands),
        ),
    )
    if not no_persist:
        report.persist(Path.cwd())
    _emit(report, compact=compact, human=human)


@app.command("report", help="Re-project the persisted last report.")
def report(
    compact: Annotated[
        bool,
        typer.Option("--compact", help="Emit one dense JSON report line."),
    ] = False,
    human: Annotated[
        bool,
        typer.Option("--human", help="Also render a short summary to stderr."),
    ] = False,
) -> None:
    """Read and emit ``.guard/last-report.json``."""
    try:
        persisted = GuardReport.read_last(Path.cwd())
    except (OSError, ValueError, KeyError) as error:
        persisted = GuardReport.tool_error(
            "report",
            ".",
            5,
            f"no readable .guard/last-report.json: {error}",
        )
    _emit(persisted, compact=compact, human=human)


@app.command("spec", help="Describe Guard's report and policy surface offline.")
def spec(
    compact: Annotated[
        bool,
        typer.Option("--compact", help="Emit one dense JSON report line."),
    ] = False,
    human: Annotated[
        bool,
        typer.Option("--human", help="Also render a short summary to stderr."),
    ] = False,
) -> None:
    """Emit the machine-readable offline contract."""
    _emit(
        GuardReport.stub(
            "spec",
            "guard.report/1: reference static/security-lint findings plus "
            "optional vat/rig/meter evidence adapters; arena is compatibility-only.",
        ),
        compact=compact,
        human=human,
    )


@app.command("llm", help="Render the offline Guard agent playbook.")
def llm(
    compact: Annotated[
        bool,
        typer.Option("--compact", help="Emit one dense JSON report line."),
    ] = False,
    human: Annotated[
        bool,
        typer.Option("--human", help="Also render a short summary to stderr."),
    ] = False,
) -> None:
    """Teach an agent to use Guard without network access."""
    _emit(
        GuardReport.stub(
            "llm",
            "Use `guard scan <path> --profile security-lint`. Add vat/rig/meter "
            "evidence flags for dynamic evidence. Treat findings as actionable "
            "unless a reviewed waiver exists.",
        ),
        compact=compact,
        human=human,
    )


def _evidence_commands(
    path: Path,
    vat_runners: list[str],
    vat_commands: list[str],
    rig_dirs: list[Path],
    rig_scenarios: list[Path],
    rig_commands: list[str],
    meter_targets: list[Path],
    meter_commands: list[str],
    arena_specs: list[Path],
    arena_commands: list[str],
) -> list[EvidenceCommand]:
    commands: list[EvidenceCommand] = []
    vat_cwd = path if path.is_dir() else path.parent
    for runner in vat_runners:
        command = EvidenceCommand.argv(
            "vat",
            runner,
            [_tool("vat"), "run", "--json", runner],
        )
        if (vat_cwd / "vat.toml").exists():
            command = command.with_cwd(vat_cwd)
        commands.append(command)
    commands.extend(
        EvidenceCommand.shell("vat", command, command)
        for command in vat_commands
    )
    for directory in rig_dirs:
        commands.append(
            EvidenceCommand.argv(
                "rig",
                str(directory),
                [_tool("rig"), "run", "--dir", str(directory), "--compact"],
            )
        )
    for scenario in rig_scenarios:
        commands.append(
            EvidenceCommand.argv(
                "rig",
                str(scenario),
                [_tool("rig"), "run", "--scenario", str(scenario), "--compact"],
            )
        )
    commands.extend(
        EvidenceCommand.shell("rig", command, command)
        for command in rig_commands
    )
    for target in meter_targets:
        commands.append(
            EvidenceCommand.argv(
                "meter",
                str(target),
                [
                    _tool("meter"),
                    "run",
                    "--target",
                    str(target),
                    "--skip-bench",
                    "--skip-profile",
                    "--compact",
                ],
            )
            .with_env("CC", "/usr/bin/cc")
            .with_env("PATH", _stable_rust_path())
        )
    commands.extend(
        EvidenceCommand.shell("meter", command, command)
        for command in meter_commands
    )
    for arena in arena_specs:
        commands.append(
            EvidenceCommand.argv(
                "arena",
                str(arena),
                [_tool("arena"), "run", "--spec", str(arena), "--compact"],
            )
        )
    commands.extend(
        EvidenceCommand.shell("arena", command, command)
        for command in arena_commands
    )
    return commands


def _tool(name: str) -> str:
    return shutil.which(name) or name


def _stable_rust_path() -> str:
    rustup = str(Path.home() / ".rustup/toolchains/stable-aarch64-apple-darwin/bin")
    cargo = str(Path.home() / ".cargo/bin")
    return ":".join(
        (
            rustup,
            "/opt/homebrew/bin",
            "/usr/local/bin",
            "/usr/bin",
            "/bin",
            "/usr/sbin",
            "/sbin",
            cargo,
            os.environ.get("PATH", ""),
        )
    )


def _emit(
    report_value: GuardReport,
    compact: bool = False,
    human: bool = False,
) -> None:
    output = OutputOptions(_OUTPUT.compact or compact, _OUTPUT.human or human)
    typer.echo(report_value.to_json(output.compact))
    if output.human:
        typer.echo(
            f"guard {report_value.verb} -> exit {report_value.exit_code} "
            f"(security_findings={report_value.summary.security_findings})",
            err=True,
        )
    if report_value.exit_code:
        raise typer.Exit(report_value.exit_code)


if __name__ == "__main__":
    app()
