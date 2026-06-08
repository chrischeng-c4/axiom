# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "errors"
# case = "bad_conflict_handler_raises"
# subject = "argparse.ArgumentParser"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""argparse.ArgumentParser: bad_conflict_handler_raises (errors)."""
import argparse

_raised = False
try:
    argparse.ArgumentParser(conflict_handler='nope')
except ValueError:
    _raised = True
assert _raised, "bad_conflict_handler_raises: expected ValueError"
print("bad_conflict_handler_raises OK")
