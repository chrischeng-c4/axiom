# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "errors"
# case = "nargs_plus_empty_exits"
# subject = "argparse.ArgumentParser.parse_args"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""argparse.ArgumentParser.parse_args: a nargs='+' positional with zero supplied values raises SystemExit, but a single value parses"""
import argparse
import contextlib
import io

p = argparse.ArgumentParser()
p.add_argument("items", nargs="+")
ns = p.parse_args(["x"])
assert ns.items == ["x"], f"nargs=+ one = {ns.items!r}"
_raised = False
with contextlib.redirect_stderr(io.StringIO()):
    try:
        p.parse_args([])
    except SystemExit:
        _raised = True
assert _raised, "nargs=+ empty raises SystemExit"
print("nargs_plus_empty_exits OK")
