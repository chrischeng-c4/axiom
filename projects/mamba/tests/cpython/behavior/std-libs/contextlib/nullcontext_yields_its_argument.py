# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "behavior"
# case = "nullcontext_yields_its_argument"
# subject = "contextlib.nullcontext"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_contextlib.py"
# status = "filled"
# ///
"""contextlib.nullcontext: nullcontext(value) yields the value to `as` and is a no-op manager; nullcontext() with no argument yields None"""
import contextlib

with contextlib.nullcontext("token") as tok:
    assert tok == "token", f"nullcontext value = {tok!r}"

with contextlib.nullcontext(42) as n:
    assert n == 42, f"nullcontext value = {n!r}"

with contextlib.nullcontext() as none:
    assert none is None, f"nullcontext() with no arg = {none!r}"

print("nullcontext_yields_its_argument OK")
