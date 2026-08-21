# test_click.py — #3459 axis-1 3p click AssertionPass seed.
#
# Mamba-authored seed exercising the click CLI-framework surface
# called out in the issue:
#   * click.Command + Option + Argument + Group
#   * type coercion (INT / FLOAT / BOOL / CHOICE)
#   * default values
#   * multiple=True
#   * BadParameter on invalid input
#
# Contract placement: `spec/` — pins outcome Fail. Mamba runtime gap
# (pkgmgr cannot yet install pure-Python wheels per #1262 Phase 1.5)
# blocks `import click` on mamba today. Once mamba pkgmgr installs
# click cleanly and the seed flips to AssertionPass on mamba, drift
# detection prompts a `git mv spec/test_click.py pass/test_click.py`.
#
# Surface coverage (asserts run at module scope; no helper closures per
# the mamba top-level def() quirk in test_math.py):
#   1. Module identity + version surface.
#   2. click.Command — name, params, callback wiring; CliRunner invokes
#      command and captures output.
#   3. click.Option — type coercion (INT, FLOAT) and default values.
#   4. click.Argument — required positional captures and forwards.
#   5. Option(multiple=True) — collects repeated values into a tuple.
#   6. Choice — restricts values; BadParameter raised for invalid input.
#   7. click.Group — sub-command dispatch; runner.invoke routes correctly.
#
# Contract with `cpython_lib_test_runner.rs`:
#   - Each `assert` runs at top level. Inverting any assert raises
#     AssertionError → non-zero exit → runner classifies as `Fail`.
#   - After every assertion has executed, the seed emits
#     `MAMBA_ASSERTION_PASS: test_click N asserts` to stdout.

import click
from click.testing import CliRunner

_ledger: list[int] = []

# 1. Module identity.
assert click.__name__ == "click", "click.__name__"
_ledger.append(1)
assert hasattr(click, "Command"), "click exposes Command"
_ledger.append(1)
assert hasattr(click, "Group"), "click exposes Group"
_ledger.append(1)
assert hasattr(click, "Option"), "click exposes Option"
_ledger.append(1)
assert hasattr(click, "Argument"), "click exposes Argument"
_ledger.append(1)


# 2. click.Command — basic invocation via CliRunner.
@click.command()
@click.option("--name", default="world", help="who to greet")
def _greet(name: str) -> None:
    click.echo(f"hello {name}")


_runner = CliRunner()
_res = _runner.invoke(_greet, [])
assert _res.exit_code == 0, "default greet exits 0"
_ledger.append(1)
assert _res.output == "hello world\n", "default greet output uses default name"
_ledger.append(1)
_res2 = _runner.invoke(_greet, ["--name", "alice"])
assert _res2.exit_code == 0, "named greet exits 0"
_ledger.append(1)
assert _res2.output == "hello alice\n", "named greet uses passed name"
_ledger.append(1)


# 3. Option type coercion — INT and FLOAT.
@click.command()
@click.option("--n", type=int, default=0)
@click.option("--f", type=float, default=0.0)
def _coerce(n: int, f: float) -> None:
    click.echo(f"{n}|{f}")


_res3 = _runner.invoke(_coerce, ["--n", "7", "--f", "1.5"])
assert _res3.exit_code == 0, "coerce exits 0"
_ledger.append(1)
assert _res3.output == "7|1.5\n", "INT and FLOAT type coercion produces native types"
_ledger.append(1)
# Default values used when option omitted.
_res4 = _runner.invoke(_coerce, [])
assert _res4.exit_code == 0, "coerce-default exits 0"
_ledger.append(1)
assert _res4.output == "0|0.0\n", "Option defaults applied when arg omitted"
_ledger.append(1)


# 4. click.Argument — positional required.
@click.command()
@click.argument("target")
def _shout(target: str) -> None:
    click.echo(target.upper())


_res5 = _runner.invoke(_shout, ["hi"])
assert _res5.exit_code == 0, "shout 'hi' exits 0"
_ledger.append(1)
assert _res5.output == "HI\n", "argument captured and forwarded"
_ledger.append(1)
# Missing required argument → non-zero exit.
_res6 = _runner.invoke(_shout, [])
assert _res6.exit_code != 0, "missing required argument fails"
_ledger.append(1)


# 5. Option(multiple=True) — collects repeated values.
@click.command()
@click.option("--item", multiple=True)
def _collect(item: tuple) -> None:
    click.echo("|".join(item))


_res7 = _runner.invoke(_collect, ["--item", "a", "--item", "b", "--item", "c"])
assert _res7.exit_code == 0, "collect exits 0"
_ledger.append(1)
assert _res7.output == "a|b|c\n", "multiple=True collects repeated values in order"
_ledger.append(1)
# Empty list when option omitted.
_res8 = _runner.invoke(_collect, [])
assert _res8.exit_code == 0, "empty collect exits 0"
_ledger.append(1)
assert _res8.output == "\n", "multiple=True yields empty tuple when not passed"
_ledger.append(1)


# 6. Choice — restricts values; invalid input → non-zero exit + BadParameter
# message in stderr.
@click.command()
@click.option("--mode", type=click.Choice(["fast", "slow"]), default="fast")
def _modey(mode: str) -> None:
    click.echo(mode)


_res9 = _runner.invoke(_modey, ["--mode", "fast"])
assert _res9.exit_code == 0, "fast choice exits 0"
_ledger.append(1)
assert _res9.output == "fast\n", "Choice accepts valid value"
_ledger.append(1)
_res10 = _runner.invoke(_modey, ["--mode", "bogus"])
assert _res10.exit_code != 0, "invalid choice fails"
_ledger.append(1)
# click writes the BadParameter / UsageError diagnostic into output by default
# when stdin/stderr are merged via CliRunner.
assert "bogus" in _res10.output, "BadParameter mentions the invalid value"
_ledger.append(1)


# 7. click.Group — sub-command dispatch.
@click.group()
def _cli() -> None:
    pass


@_cli.command("add")  # type: ignore[attr-defined]
@click.argument("a", type=int)
@click.argument("b", type=int)
def _add(a: int, b: int) -> None:
    click.echo(str(a + b))


@_cli.command("sub")  # type: ignore[attr-defined]
@click.argument("a", type=int)
@click.argument("b", type=int)
def _sub(a: int, b: int) -> None:
    click.echo(str(a - b))


_res_add = _runner.invoke(_cli, ["add", "3", "4"])
assert _res_add.exit_code == 0, "group add exits 0"
_ledger.append(1)
assert _res_add.output == "7\n", "group dispatch routes to add subcommand"
_ledger.append(1)
_res_sub = _runner.invoke(_cli, ["sub", "10", "4"])
assert _res_sub.exit_code == 0, "group sub exits 0"
_ledger.append(1)
assert _res_sub.output == "6\n", "group dispatch routes to sub subcommand"
_ledger.append(1)
# Unknown subcommand → non-zero exit.
_res_bad = _runner.invoke(_cli, ["bogus"])
assert _res_bad.exit_code != 0, "unknown subcommand fails"
_ledger.append(1)


# Emit the proof-of-execution marker. Per `ASSERTION_PASS_MARKERS` in
# `cpython_lib_test_runner.rs`, presence of this token escalates the
# outcome from ImportPass to AssertionPass.
print(f"MAMBA_ASSERTION_PASS: test_click {len(_ledger)} asserts")
