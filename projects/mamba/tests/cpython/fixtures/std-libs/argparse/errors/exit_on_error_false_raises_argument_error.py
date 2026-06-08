# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "errors"
# case = "exit_on_error_false_raises_argument_error"
# subject = "argparse.ArgumentParser"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""argparse.ArgumentParser: with exit_on_error=False a parse failure (bad int value) raises ArgumentError instead of printing to stderr and exiting; good args still parse"""
import argparse

p = argparse.ArgumentParser(exit_on_error=False)
p.add_argument("--integers", metavar="N", type=int)

# Good args still parse normally.
ns = p.parse_args(["--integers", "4"])
assert ns.integers == 4, f"exit_on_error good = {ns.integers!r}"

# A bad value raises ArgumentError instead of SystemExit.
_raised = False
try:
    p.parse_args(["--integers", "a"])
except argparse.ArgumentError:
    _raised = True
assert _raised, "exit_on_error=False raises ArgumentError on bad value"
print("exit_on_error_false_raises_argument_error OK")
