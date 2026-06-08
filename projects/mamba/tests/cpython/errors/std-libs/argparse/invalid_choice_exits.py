# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "errors"
# case = "invalid_choice_exits"
# subject = "argparse.ArgumentParser.parse_args"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""argparse.ArgumentParser.parse_args: parse_args raises SystemExit when a value is not in the declared choices= set, but accepts a valid choice"""
import argparse
import contextlib
import io

p = argparse.ArgumentParser()
p.add_argument("--mode", choices=["fast", "slow"])
ns = p.parse_args(["--mode", "fast"])
assert ns.mode == "fast", f"valid choice = {ns.mode!r}"
_raised = False
with contextlib.redirect_stderr(io.StringIO()):
    try:
        p.parse_args(["--mode", "invalid"])
    except SystemExit:
        _raised = True
assert _raised, "invalid choice raises SystemExit"
print("invalid_choice_exits OK")
