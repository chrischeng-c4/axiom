# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "errors"
# case = "duplicate_subparser_name_raises"
# subject = "argparse.ArgumentParser"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""argparse.ArgumentParser: duplicate_subparser_name_raises (errors)."""
import argparse

_raised = False
try:
    _p = argparse.ArgumentParser(); _sp = _p.add_subparsers(); _sp.add_parser('build'); _sp.add_parser('build')
except argparse.ArgumentError:
    _raised = True
assert _raised, "duplicate_subparser_name_raises: expected argparse.ArgumentError"
print("duplicate_subparser_name_raises OK")
