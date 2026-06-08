# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "security-matrix"
# lib = "addr"
# dimension = "security"
# case = "RuntimeError__addr"
# subject = "RuntimeError message must not leak memory address"
# kind = "semantic"
# xfail = "error-message leak check; mamba must not surface memory address in RuntimeError messages"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
# mamba-xfail: error-message leak check; mamba must not surface memory address in RuntimeError messages
"""Safety wall (addr): RuntimeError message must not leak a memory address.

Triggers RuntimeError and scans its message for an internal memory address. mamba must keep
error messages free of internals; LEAK marks a message the keep must sanitize."""

import re

PATTERN = r"0x[0-9a-fA-F]{6,}"
try:
    raise RuntimeError("runtime condition")
    raised = None
except BaseException as e:
    raised = e

msg = "" if raised is None else (str(raised) + " " + repr(raised))
print("LEAK:addr" if re.search(PATTERN, msg) else "safe:addr")
