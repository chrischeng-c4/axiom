"""Typer surface over the aw engine scripts.

Each subcommand rebuilds the argv its engine module already parses and hands
it to that module's ``main(argv)``. argparse stays the single source of
validation — the options here are deliberately plain ``str`` so a registry
change in ``wi_types`` never has to be mirrored into a typer Enum.
"""

from __future__ import annotations

import importlib
import sys
from pathlib import Path

import typer

from aw import __version__

_SCRIPTS = Path(__file__).resolve().parent / "scripts"

app = typer.Typer(no_args_is_help=True, help="aw workflow CLI")
change_app = typer.Typer(no_args_is_help=True, help="Typed delivery issues on the tracker.")
milestone_app = typer.Typer(no_args_is_help=True, help="Release Milestones on the tracker.")
e2e_app = typer.Typer(no_args_is_help=True, help="Behavior phase 1: the black-box contract.")
impl_app = typer.Typer(no_args_is_help=True, help="Behavior phase 2: skeleton, red tests, implementation.")
maint_app = typer.Typer(no_args_is_help=True, help="Maintenance phase for refactor/test/docs/chore issues.")
wis_app = typer.Typer(no_args_is_help=True, help="Promise-to-tracker gap measurement.")
meta_app = typer.Typer(no_args_is_help=True, help="META-doc rule scan (read-only).")
metadoc_app = typer.Typer(no_args_is_help=True, help="META-doc write allowlist check and commit.")

app.add_typer(change_app, name="change")
app.add_typer(milestone_app, name="milestone")
app.add_typer(e2e_app, name="e2e")
app.add_typer(impl_app, name="impl")
app.add_typer(maint_app, name="maint")
app.add_typer(wis_app, name="wis")
app.add_typer(meta_app, name="meta")
app.add_typer(metadoc_app, name="metadoc")


def _delegate(module: str, argv: list[str]) -> None:
    """Run one engine module's argparse ``main`` with a rebuilt argv."""
    scripts = str(_SCRIPTS)
    if scripts not in sys.path:
        sys.path.insert(0, scripts)
    mod = importlib.import_module(module)
    raise typer.Exit(mod.main(argv))


@app.callback()
def main() -> None:
    """aw workflow CLI."""


@app.command()
def version() -> None:
    """Print the aw version."""
    typer.echo(__version__)


# --- change ---------------------------------------------------------------

@change_app.callback()
def change_main(
    ctx: typer.Context,
    repo: str | None = typer.Option(None, "--repo", help="owner/repo override."),
) -> None:
    ctx.obj = ["--repo", repo] if repo else []


@change_app.command("skeleton")
def change_skeleton(
    ctx: typer.Context,
    type_: str = typer.Option(..., "--type", help="Delivery issue type."),
) -> None:
    """Print the GHAN body skeleton for one type."""
    _delegate("change", [*ctx.obj, "skeleton", "--type", type_])


@change_app.command("bodydir")
def change_bodydir(
    ctx: typer.Context,
    type_: str = typer.Option(..., "--type", help="Delivery issue type."),
) -> None:
    """Print the staged-body directory for one type."""
    _delegate("change", [*ctx.obj, "bodydir", "--type", type_])


@change_app.command("fetch")
def change_fetch(ctx: typer.Context, iid: str = typer.Argument(...)) -> None:
    """Fetch one issue body from the tracker into the staging area."""
    _delegate("change", [*ctx.obj, "fetch", iid])


@change_app.command("adopt")
def change_adopt(
    ctx: typer.Context,
    path: str = typer.Argument(...),
    iid: str = typer.Argument(...),
    type_: str = typer.Option(..., "--type", help="Delivery issue type."),
) -> None:
    """Adopt a local body file as the staged body for one issue."""
    _delegate("change", [*ctx.obj, "adopt", path, iid, "--type", type_])


@change_app.command("validate")
def change_validate(
    ctx: typer.Context,
    iid: str | None = typer.Argument(None),
    body_file: str | None = typer.Option(None, "--body-file"),
    type_: str | None = typer.Option(None, "--type"),
    json_: bool = typer.Option(False, "--json"),
) -> None:
    """Validate a GHAN body (staged, tracked, or a local file)."""
    argv = [*ctx.obj, "validate"]
    if iid is not None:
        argv.append(iid)
    if body_file is not None:
        argv += ["--body-file", body_file]
    if type_ is not None:
        argv += ["--type", type_]
    if json_:
        argv.append("--json")
    _delegate("change", argv)


@change_app.command("show")
def change_show(
    ctx: typer.Context,
    iid: str = typer.Argument(...),
    json_: bool = typer.Option(False, "--json"),
) -> None:
    """Show one issue's tracker state."""
    argv = [*ctx.obj, "show", iid]
    if json_:
        argv.append("--json")
    _delegate("change", argv)


@change_app.command("create")
def change_create(
    ctx: typer.Context,
    title: str = typer.Option(..., "--title"),
    body_file: str = typer.Option(..., "--body-file"),
    type_: str = typer.Option(..., "--type"),
    milestone: str | None = typer.Option(None, "--milestone"),
    priority: str = typer.Option("p2", "--priority"),
    project: str | None = typer.Option(None, "--project"),
    dry_run: bool = typer.Option(False, "--dry-run"),
) -> None:
    """Create one typed delivery issue from a validated body."""
    argv = [*ctx.obj, "create", "--title", title, "--body-file", body_file,
            "--type", type_, "--priority", priority]
    if milestone is not None:
        argv += ["--milestone", milestone]
    if project is not None:
        argv += ["--project", project]
    if dry_run:
        argv.append("--dry-run")
    _delegate("change", argv)


@change_app.command("lifecycle")
def change_lifecycle(
    ctx: typer.Context,
    iid: str = typer.Argument(...),
    leg: str = typer.Option(..., "--leg"),
    commit: str = typer.Option(..., "--commit"),
    digest: str = typer.Option(..., "--digest"),
    dry_run: bool = typer.Option(False, "--dry-run"),
) -> None:
    """Record one landed phase commit on the issue."""
    argv = [*ctx.obj, "lifecycle", iid, "--leg", leg,
            "--commit", commit, "--digest", digest]
    if dry_run:
        argv.append("--dry-run")
    _delegate("change", argv)


@change_app.command("update")
def change_update(
    ctx: typer.Context,
    iid: str = typer.Argument(...),
    body_file: str | None = typer.Option(None, "--body-file"),
    title: str | None = typer.Option(None, "--title"),
    add_label: list[str] = typer.Option([], "--add-label"),
    remove_label: list[str] = typer.Option([], "--remove-label"),
    milestone: str | None = typer.Option(None, "--milestone"),
    remove_milestone: bool = typer.Option(False, "--remove-milestone"),
    dry_run: bool = typer.Option(False, "--dry-run"),
) -> None:
    """Update one issue's body, title, labels, or Milestone binding."""
    argv = [*ctx.obj, "update", iid]
    if body_file is not None:
        argv += ["--body-file", body_file]
    if title is not None:
        argv += ["--title", title]
    for label in add_label:
        argv += ["--add-label", label]
    for label in remove_label:
        argv += ["--remove-label", label]
    if milestone is not None:
        argv += ["--milestone", milestone]
    if remove_milestone:
        argv.append("--remove-milestone")
    if dry_run:
        argv.append("--dry-run")
    _delegate("change", argv)


@change_app.command("retype")
def change_retype(
    ctx: typer.Context,
    iid: str = typer.Argument(...),
    to: str = typer.Option(..., "--to"),
    dry_run: bool = typer.Option(False, "--dry-run"),
) -> None:
    """Change one issue's delivery type."""
    argv = [*ctx.obj, "retype", iid, "--to", to]
    if dry_run:
        argv.append("--dry-run")
    _delegate("change", argv)


@change_app.command("close")
def change_close(
    ctx: typer.Context,
    iid: str = typer.Argument(...),
    dry_run: bool = typer.Option(False, "--dry-run"),
) -> None:
    """Close one issue once its evidence is terminal."""
    argv = [*ctx.obj, "close", iid]
    if dry_run:
        argv.append("--dry-run")
    _delegate("change", argv)


# --- milestone ------------------------------------------------------------

@milestone_app.callback()
def milestone_main(
    ctx: typer.Context,
    repo: str | None = typer.Option(None, "--repo", help="owner/repo override."),
) -> None:
    ctx.obj = ["--repo", repo] if repo else []


@milestone_app.command("skeleton")
def milestone_skeleton(ctx: typer.Context) -> None:
    """Print the release Milestone description skeleton."""
    _delegate("milestone", [*ctx.obj, "skeleton"])


@milestone_app.command("validate")
def milestone_validate(
    ctx: typer.Context,
    ref: str | None = typer.Argument(None),
    description_file: str | None = typer.Option(None, "--description-file"),
    title: str | None = typer.Option(None, "--title"),
    draft: bool = typer.Option(False, "--draft"),
    json_: bool = typer.Option(False, "--json"),
) -> None:
    """Validate a Milestone description (tracked or a local file)."""
    argv = [*ctx.obj, "validate"]
    if ref is not None:
        argv.append(ref)
    if description_file is not None:
        argv += ["--description-file", description_file]
    if title is not None:
        argv += ["--title", title]
    if draft:
        argv.append("--draft")
    if json_:
        argv.append("--json")
    _delegate("milestone", argv)


def _milestone_ref_verb(ctx: typer.Context, verb: str, ref: str, json_: bool) -> None:
    argv = [*ctx.obj, verb, ref]
    if json_:
        argv.append("--json")
    _delegate("milestone", argv)


@milestone_app.command("show")
def milestone_show(
    ctx: typer.Context,
    ref: str = typer.Argument(...),
    json_: bool = typer.Option(False, "--json"),
) -> None:
    """Show one release Milestone."""
    _milestone_ref_verb(ctx, "show", ref, json_)


@milestone_app.command("children")
def milestone_children(
    ctx: typer.Context,
    ref: str = typer.Argument(...),
    json_: bool = typer.Option(False, "--json"),
) -> None:
    """List the delivery issues assigned to one Milestone."""
    _milestone_ref_verb(ctx, "children", ref, json_)


@milestone_app.command("reconcile")
def milestone_reconcile(
    ctx: typer.Context,
    ref: str = typer.Argument(...),
    json_: bool = typer.Option(False, "--json"),
) -> None:
    """Check one Milestone's assigned set against its Development Order."""
    _milestone_ref_verb(ctx, "reconcile", ref, json_)


@milestone_app.command("order")
def milestone_order(
    ctx: typer.Context,
    ref: str = typer.Argument(...),
    open_only: bool = typer.Option(False, "--open-only"),
    json_: bool = typer.Option(False, "--json"),
) -> None:
    """Print one Milestone's Development Order."""
    argv = [*ctx.obj, "order", ref]
    if open_only:
        argv.append("--open-only")
    if json_:
        argv.append("--json")
    _delegate("milestone", argv)


@milestone_app.command("next")
def milestone_next(
    ctx: typer.Context,
    ref: str = typer.Argument(...),
    json_: bool = typer.Option(False, "--json"),
) -> None:
    """Select one Milestone's executable queue head."""
    _milestone_ref_verb(ctx, "next", ref, json_)


@milestone_app.command("versions")
def milestone_versions(
    ctx: typer.Context,
    project: str | None = typer.Option(None, "--project"),
    state: str = typer.Option("open", "--state"),
    json_: bool = typer.Option(False, "--json"),
) -> None:
    """List release Milestone versions."""
    argv = [*ctx.obj, "versions", "--state", state]
    if project is not None:
        argv += ["--project", project]
    if json_:
        argv.append("--json")
    _delegate("milestone", argv)


@milestone_app.command("next-version")
def milestone_next_version(
    ctx: typer.Context,
    project: str = typer.Argument(...),
    bump: str | None = typer.Option(None, "--bump"),
    json_: bool = typer.Option(False, "--json"),
) -> None:
    """Compute the next release version for one project."""
    argv = [*ctx.obj, "next-version", project]
    if bump is not None:
        argv += ["--bump", bump]
    if json_:
        argv.append("--json")
    _delegate("milestone", argv)


@milestone_app.command("create")
def milestone_create(
    ctx: typer.Context,
    title: str = typer.Option(..., "--title"),
    description_file: str = typer.Option(..., "--description-file"),
    due_on: str | None = typer.Option(None, "--due-on"),
    draft: bool = typer.Option(False, "--draft"),
    dry_run: bool = typer.Option(False, "--dry-run"),
) -> None:
    """Create one release Milestone from a validated description."""
    argv = [*ctx.obj, "create", "--title", title,
            "--description-file", description_file]
    if due_on is not None:
        argv += ["--due-on", due_on]
    if draft:
        argv.append("--draft")
    if dry_run:
        argv.append("--dry-run")
    _delegate("milestone", argv)


@milestone_app.command("update")
def milestone_update(
    ctx: typer.Context,
    ref: str = typer.Argument(...),
    title: str | None = typer.Option(None, "--title"),
    description_file: str | None = typer.Option(None, "--description-file"),
    due_on: str | None = typer.Option(None, "--due-on"),
    clear_due_on: bool = typer.Option(False, "--clear-due-on"),
    draft: bool = typer.Option(False, "--draft"),
    dry_run: bool = typer.Option(False, "--dry-run"),
) -> None:
    """Update one release Milestone."""
    argv = [*ctx.obj, "update", ref]
    if title is not None:
        argv += ["--title", title]
    if description_file is not None:
        argv += ["--description-file", description_file]
    if due_on is not None:
        argv += ["--due-on", due_on]
    if clear_due_on:
        argv.append("--clear-due-on")
    if draft:
        argv.append("--draft")
    if dry_run:
        argv.append("--dry-run")
    _delegate("milestone", argv)


@milestone_app.command("close")
def milestone_close(
    ctx: typer.Context,
    ref: str = typer.Argument(...),
    dry_run: bool = typer.Option(False, "--dry-run"),
) -> None:
    """Close one release Milestone once every assigned issue is terminal."""
    argv = [*ctx.obj, "close", ref]
    if dry_run:
        argv.append("--dry-run")
    _delegate("milestone", argv)


# --- e2e / impl / maint ---------------------------------------------------

def _phase_callback(ctx: typer.Context, project: str) -> None:
    ctx.obj = ["--project", project]


@e2e_app.callback()
def e2e_main(
    ctx: typer.Context,
    project: str = typer.Option(..., "--project", help="apps/<name> project."),
) -> None:
    _phase_callback(ctx, project)


@impl_app.callback()
def impl_main(
    ctx: typer.Context,
    project: str = typer.Option(..., "--project", help="apps/<name> project."),
) -> None:
    _phase_callback(ctx, project)


@maint_app.callback()
def maint_main(
    ctx: typer.Context,
    project: str = typer.Option(..., "--project", help="apps/<name> project."),
) -> None:
    _phase_callback(ctx, project)


def _phase_verb(ctx: typer.Context, module: str, verb: str, wi: int,
                dry_run: bool = False) -> None:
    argv = [*ctx.obj, verb, str(wi)]
    if dry_run:
        argv.append("--dry-run")
    _delegate(module, argv)


@e2e_app.command("start")
def e2e_start(ctx: typer.Context, wi: int = typer.Argument(...)) -> None:
    """Freeze the issue and open the e2e phase."""
    _phase_verb(ctx, "e2e", "start", wi)


@e2e_app.command("verify")
def e2e_verify(ctx: typer.Context, wi: int = typer.Argument(...)) -> None:
    """Check the e2e write root and the named failing case."""
    _phase_verb(ctx, "e2e", "verify", wi)


@e2e_app.command("test")
def e2e_test(ctx: typer.Context, wi: int = typer.Argument(...)) -> None:
    """Run the e2e case and record its red."""
    _phase_verb(ctx, "e2e", "test", wi)


@e2e_app.command("commit")
def e2e_commit(
    ctx: typer.Context,
    wi: int = typer.Argument(...),
    dry_run: bool = typer.Option(False, "--dry-run"),
) -> None:
    """Re-run every gate and land the e2e commit."""
    _phase_verb(ctx, "e2e", "commit", wi, dry_run)


@impl_app.command("start")
def impl_start(ctx: typer.Context, wi: int = typer.Argument(...)) -> None:
    """Check the e2e evidence and open the impl phase."""
    _phase_verb(ctx, "impl", "start", wi)


@impl_app.command("red")
def impl_red(ctx: typer.Context, wi: int = typer.Argument(...)) -> None:
    """Record the named failing colocated tests before implementing."""
    _phase_verb(ctx, "impl", "red", wi)


@impl_app.command("verify")
def impl_verify(ctx: typer.Context, wi: int = typer.Argument(...)) -> None:
    """Check the impl write root against the recorded red."""
    _phase_verb(ctx, "impl", "verify", wi)


@impl_app.command("test")
def impl_test(ctx: typer.Context, wi: int = typer.Argument(...)) -> None:
    """Run the suite green against the recorded red."""
    _phase_verb(ctx, "impl", "test", wi)


@impl_app.command("commit")
def impl_commit(
    ctx: typer.Context,
    wi: int = typer.Argument(...),
    dry_run: bool = typer.Option(False, "--dry-run"),
) -> None:
    """Re-run every gate and land the impl commit."""
    _phase_verb(ctx, "impl", "commit", wi, dry_run)


@maint_app.command("start")
def maint_start(ctx: typer.Context, wi: int = typer.Argument(...)) -> None:
    """Freeze the type, baseline, and change points for one maint issue."""
    _phase_verb(ctx, "maint", "start", wi)


@maint_app.command("record")
def maint_record(
    ctx: typer.Context,
    wi: int = typer.Argument(...),
    when: str = typer.Option(..., "--when", help="before or after."),
    command: str = typer.Option(..., "--command"),
    exit_code: int = typer.Option(..., "--exit"),
    output_file: str = typer.Option(..., "--output-file"),
) -> None:
    """Record one declared gate run's exact exit and output digest."""
    argv = [*ctx.obj, "record", str(wi), "--when", when, "--command", command,
            "--exit", str(exit_code), "--output-file", output_file]
    _delegate("maint", argv)


@maint_app.command("verify")
def maint_verify(ctx: typer.Context, wi: int = typer.Argument(...)) -> None:
    """Check the maint write boundary and recorded gates."""
    _phase_verb(ctx, "maint", "verify", wi)


@maint_app.command("commit")
def maint_commit(
    ctx: typer.Context,
    wi: int = typer.Argument(...),
    dry_run: bool = typer.Option(False, "--dry-run"),
) -> None:
    """Re-run every gate and land the maint commit."""
    _phase_verb(ctx, "maint", "commit", wi, dry_run)


# --- wis / meta / metadoc -------------------------------------------------

@wis_app.callback()
def wis_main() -> None:
    """Promise-to-tracker gap measurement."""


@wis_app.command("gap")
def wis_gap(
    project: str = typer.Argument(...),
    repo: str | None = typer.Option(None, "--repo"),
    format_: str = typer.Option("text", "--format"),
) -> None:
    """Print the seven-row promise/tracker gap report for one project."""
    argv = ["gap", project, "--format", format_]
    if repo is not None:
        argv += ["--repo", repo]
    _delegate("wis", argv)


@meta_app.callback()
def meta_main() -> None:
    """META-doc rule scan (read-only)."""


@meta_app.command("check")
def meta_check(
    repo: str | None = typer.Option(None, "--repo"),
    rule: list[str] = typer.Option([], "--rule"),
    path: list[str] = typer.Option([], "--path"),
    format_: str = typer.Option("text", "--format"),
) -> None:
    """Scan the tracked META-docs against the seven ratcheted rules."""
    argv = ["check", "--format", format_]
    if repo is not None:
        argv += ["--repo", repo]
    for item in rule:
        argv += ["--rule", item]
    for item in path:
        argv += ["--path", item]
    _delegate("meta", argv)


@metadoc_app.callback()
def metadoc_main() -> None:
    """META-doc write allowlist check and commit."""


@metadoc_app.command("check")
def metadoc_check(
    project: str = typer.Argument(...),
    format_: str = typer.Option("text", "--format"),
) -> None:
    """Check the dirty set against one project's META-doc allowlist."""
    _delegate("metadoc", ["check", project, "--format", format_])


@metadoc_app.command("commit")
def metadoc_commit(
    project: str = typer.Argument(...),
    why: str = typer.Option(..., "--why"),
    dry_run: bool = typer.Option(False, "--dry-run"),
) -> None:
    """Re-check, stage the allowlist, and write the trailered commit."""
    argv = ["commit", project, "--why", why]
    if dry_run:
        argv.append("--dry-run")
    _delegate("metadoc", argv)


if __name__ == "__main__":
    app()
