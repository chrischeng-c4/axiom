"""Invoke a one-option click command via `click.testing.CliRunner`.

End-user scenario: a downstream tool defines a single click command
with one `--name` option, then drives it in-process through
`CliRunner.invoke()` and asserts the captured stdout and exit code.
No terminal, no shell — CliRunner runs the parser and callback
synchronously and captures `click.echo` output.

DoD: this script must exit 0 under both CPython and mamba.
"""

import click
from click.testing import CliRunner


@click.command()
@click.option("--name", default="world", help="Name to greet.")
def greet(name):
    """Print a single greeting line."""
    click.echo(f"hello, {name}")


runner = CliRunner()

# Default option value path.
default_result = runner.invoke(greet, [])
assert default_result.exit_code == 0, f"default invocation failed: {default_result.exit_code}"
assert default_result.output == "hello, world\n", f"unexpected default output: {default_result.output!r}"

# Explicit option value path — proves --name parses end-to-end.
named_result = runner.invoke(greet, ["--name", "click"])
assert named_result.exit_code == 0, f"named invocation failed: {named_result.exit_code}"
assert named_result.output == "hello, click\n", f"unexpected named output: {named_result.output!r}"

print("ok:", named_result.output.strip())
