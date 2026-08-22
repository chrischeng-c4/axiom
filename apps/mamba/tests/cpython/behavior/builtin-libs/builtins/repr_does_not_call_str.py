# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "behavior"
# case = "repr_does_not_call_str"
# subject = "builtins.repr"
# kind = "semantic"
# xfail = ""
# status = "filled"
# ///

import re


str_calls = 0


class Boom:
    def __str__(self):
        global str_calls
        str_calls += 1
        raise ValueError("boom")


try:
    rendered = repr(Boom())
except Exception as exc:
    raise AssertionError("repr must not raise") from exc

assert str_calls == 0
assert re.sub(r"0x[0-9a-fA-F]+", "0x...", rendered) == "<__main__.Boom object at 0x...>"
