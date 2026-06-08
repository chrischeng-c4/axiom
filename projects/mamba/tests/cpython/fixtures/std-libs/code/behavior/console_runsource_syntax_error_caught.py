# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "code"
# dimension = "behavior"
# case = "console_runsource_syntax_error_caught"
# subject = "code.InteractiveConsole.runsource"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""code.InteractiveConsole.runsource: a genuine SyntaxError in source is caught and reported, not raised: ic.runsource('def bad(:') returns False with the error sent to showsyntaxerror"""
import code
import io
import contextlib

_cons = code.InteractiveConsole({})
_buf = io.StringIO()
with contextlib.redirect_stderr(_buf):
    _res = _cons.runsource("def bad(:")
# A real SyntaxError (not an incomplete continuation) is reported via
# showsyntaxerror and runsource returns False rather than propagating.
assert _res is False, f"syntax error -> False, got {_res!r}"
assert "SyntaxError" in _buf.getvalue(), "SyntaxError reported on stderr"

print("console_runsource_syntax_error_caught OK")
