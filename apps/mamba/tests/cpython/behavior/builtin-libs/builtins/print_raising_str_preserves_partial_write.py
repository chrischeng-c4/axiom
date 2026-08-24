# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "behavior"
# case = "print_raising_str_preserves_partial_write"
# subject = "builtins.print"
# kind = "semantic"
# xfail = ""
# status = "filled"
# ///

import contextlib
import io


class Boom:
    def __str__(self):
        raise ValueError("boom")


captured = io.StringIO()
caught = None
try:
    with contextlib.redirect_stdout(captured):
        print("prefix", Boom())
except ValueError as exc:
    caught = exc

assert type(caught) is ValueError
assert caught.args == ("boom",)
assert str(caught) == "boom"
assert captured.getvalue() == "prefix "
