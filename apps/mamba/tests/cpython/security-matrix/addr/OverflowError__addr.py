# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "security-matrix"
# lib = "addr"
# dimension = "security"
# case = "OverflowError__addr"
# subject = "OverflowError message must not leak memory address"
# kind = "semantic"
# xfail = "error-message leak check; mamba must not surface memory address in OverflowError messages"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
# mamba-xfail: error-message leak check; mamba must not surface memory address in OverflowError messages
"""Safety wall (addr): OverflowError message must not leak a memory address.

Triggers OverflowError and scans its message for an internal memory address. mamba must keep
error messages free of internals; LEAK marks a message the keep must sanitize."""

import re

PATTERN = r"0x[0-9a-fA-F]{6,}"
try:
    __import__('math').exp(10000)
    raised = None
except BaseException as e:
    raised = e

msg = "" if raised is None else (str(raised) + " " + repr(raised))
print("LEAK:addr" if re.search(PATTERN, msg) else "safe:addr")
