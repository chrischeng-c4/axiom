# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "errors"
# case = "required_argument_missing_exits"
# subject = "argparse.ArgumentParser.parse_args"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""argparse.ArgumentParser.parse_args: parse_args raises SystemExit when a required=True option is absent from the argument vector"""
import argparse
import contextlib
import io

p = argparse.ArgumentParser()
p.add_argument("--required-arg", required=True)
_raised = False
with contextlib.redirect_stderr(io.StringIO()):
    try:
        p.parse_args([])
    except SystemExit:
        _raised = True
assert _raised, "required argument missing raises SystemExit"
print("required_argument_missing_exits OK")
