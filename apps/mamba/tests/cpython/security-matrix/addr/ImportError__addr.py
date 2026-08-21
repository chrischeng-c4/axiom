# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "security-matrix"
# lib = "addr"
# dimension = "security"
# case = "ImportError__addr"
# subject = "ImportError message must not leak memory address"
# kind = "semantic"
# xfail = "error-message leak check; mamba must not surface memory address in ImportError messages"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
# mamba-xfail: error-message leak check; mamba must not surface memory address in ImportError messages
"""Safety wall (addr): ImportError message must not leak a memory address.

Triggers ImportError and scans its message for an internal memory address. mamba must keep
error messages free of internals; LEAK marks a message the keep must sanitize."""

import re

PATTERN = r"0x[0-9a-fA-F]{6,}"
try:
    raise ImportError("error condition")
    raised = None
except BaseException as e:
    raised = e

msg = "" if raised is None else (str(raised) + " " + repr(raised))
print("LEAK:addr" if re.search(PATTERN, msg) else "safe:addr")
