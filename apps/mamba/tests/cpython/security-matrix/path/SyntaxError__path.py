# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "security-matrix"
# lib = "path"
# dimension = "security"
# case = "SyntaxError__path"
# subject = "SyntaxError message must not leak filesystem path"
# kind = "semantic"
# xfail = "error-message leak check; mamba must not surface filesystem path in SyntaxError messages"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
# mamba-xfail: error-message leak check; mamba must not surface filesystem path in SyntaxError messages
"""Safety wall (path): SyntaxError message must not leak a filesystem path.

Triggers SyntaxError and scans its message for an internal filesystem path. mamba must keep
error messages free of internals; LEAK marks a message the keep must sanitize."""

import re

PATTERN = r"/(?:Users|home|private|tmp|var|opt|usr)/\S+"
try:
    raise SyntaxError("error condition")
    raised = None
except BaseException as e:
    raised = e

msg = "" if raised is None else (str(raised) + " " + repr(raised))
print("LEAK:path" if re.search(PATTERN, msg) else "safe:path")
