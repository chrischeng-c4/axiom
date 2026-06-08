# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "errors"
# case = "unknown_option_exits"
# subject = "argparse.ArgumentParser.parse_args"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""argparse.ArgumentParser.parse_args: parse_args raises SystemExit (status 2) on an unrecognized option, captured with stderr redirected"""
import argparse
import contextlib
import io

p = argparse.ArgumentParser(prog="prog")
p.add_argument("--n", type=int)
_code = None
with contextlib.redirect_stderr(io.StringIO()):
    try:
        p.parse_args(["--unknown"])
    except SystemExit as e:
        _code = e.code
assert _code == 2, f"unknown option exit code = {_code!r}"
print("unknown_option_exits OK")
