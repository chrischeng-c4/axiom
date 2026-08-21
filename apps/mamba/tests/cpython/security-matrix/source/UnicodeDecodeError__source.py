# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "security-matrix"
# lib = "source"
# dimension = "security"
# case = "UnicodeDecodeError__source"
# subject = "UnicodeDecodeError message must not leak source snippet"
# kind = "semantic"
# xfail = "error-message leak check; mamba must not surface source snippet in UnicodeDecodeError messages"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
# mamba-xfail: error-message leak check; mamba must not surface source snippet in UnicodeDecodeError messages
"""Safety wall (source): UnicodeDecodeError message must not leak a source snippet.

Triggers UnicodeDecodeError and scans its message for an internal source snippet. mamba must keep
error messages free of internals; LEAK marks a message the keep must sanitize."""

import re

PATTERN = r"\.rs:\d+|/src/|File \"[^\"]+\", line \d+"
try:
    b"\xff\xfe".decode("utf-8")
    raised = None
except BaseException as e:
    raised = e

msg = "" if raised is None else (str(raised) + " " + repr(raised))
print("LEAK:source" if re.search(PATTERN, msg) else "safe:source")
