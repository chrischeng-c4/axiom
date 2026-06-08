# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "errors"
# case = "duplicate_option_string_raises"
# subject = "argparse.ArgumentParser"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""argparse.ArgumentParser: duplicate_option_string_raises (errors)."""
import argparse

_raised = False
try:
    _p = argparse.ArgumentParser(); _p.add_argument('--flag'); _p.add_argument('--flag')
except argparse.ArgumentError:
    _raised = True
assert _raised, "duplicate_option_string_raises: expected argparse.ArgumentError"
print("duplicate_option_string_raises OK")
