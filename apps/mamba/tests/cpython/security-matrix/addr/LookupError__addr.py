# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "security-matrix"
# lib = "addr"
# dimension = "security"
# case = "LookupError__addr"
# subject = "LookupError message must not leak memory address"
# kind = "semantic"
# xfail = "error-message leak check; mamba must not surface memory address in LookupError messages"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
# mamba-xfail: error-message leak check; mamba must not surface memory address in LookupError messages
"""Safety wall (addr): LookupError message must not leak a memory address.

Triggers LookupError and scans its message for an internal memory address. mamba must keep
error messages free of internals; LEAK marks a message the keep must sanitize."""

import re

PATTERN = r"0x[0-9a-fA-F]{6,}"
try:
    raise LookupError("error condition")
    raised = None
except BaseException as e:
    raised = e

msg = "" if raised is None else (str(raised) + " " + repr(raised))
print("LEAK:addr" if re.search(PATTERN, msg) else "safe:addr")
