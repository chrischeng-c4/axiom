# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "security-matrix"
# lib = "source"
# dimension = "security"
# case = "ArithmeticError__source"
# subject = "ArithmeticError message must not leak source snippet"
# kind = "semantic"
# xfail = "error-message leak check; mamba must not surface source snippet in ArithmeticError messages"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
# mamba-xfail: error-message leak check; mamba must not surface source snippet in ArithmeticError messages
"""Safety wall (source): ArithmeticError message must not leak a source snippet.

Triggers ArithmeticError and scans its message for an internal source snippet. mamba must keep
error messages free of internals; LEAK marks a message the keep must sanitize."""

import re

PATTERN = r"\.rs:\d+|/src/|File \"[^\"]+\", line \d+"
try:
    raise ArithmeticError("error condition")
    raised = None
except BaseException as e:
    raised = e

msg = "" if raised is None else (str(raised) + " " + repr(raised))
print("LEAK:source" if re.search(PATTERN, msg) else "safe:source")
