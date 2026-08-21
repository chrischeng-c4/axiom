# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "errors"
# case = "argument_type_error_becomes_systemexit"
# subject = "argparse.ArgumentTypeError"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""argparse.ArgumentTypeError: a custom type= callable raising ArgumentTypeError is intercepted by argparse and re-raised as SystemExit (status 2), while a valid value parses cleanly"""
import argparse
import contextlib
import io


def positive_int(s: str) -> int:
    v = int(s)
    if v <= 0:
        raise argparse.ArgumentTypeError(f"{s!r} is not positive")
    return v


p = argparse.ArgumentParser(prog="prog")
p.add_argument("--n", type=positive_int)

# Valid value parses cleanly.
ns = p.parse_args(["--n", "5"])
assert ns.n == 5, f"valid value = {ns.n!r}"

# Invalid value: argparse intercepts ArgumentTypeError, raises SystemExit(2).
_code = None
with contextlib.redirect_stderr(io.StringIO()):
    try:
        p.parse_args(["--n", "-3"])
    except SystemExit as e:
        _code = e.code
assert _code == 2, f"ArgumentTypeError exit code = {_code!r}"
print("argument_type_error_becomes_systemexit OK")
