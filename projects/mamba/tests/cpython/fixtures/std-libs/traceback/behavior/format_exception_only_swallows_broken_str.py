# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "behavior"
# case = "format_exception_only_swallows_broken_str"
# subject = "traceback.format_exception_only"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""traceback.format_exception_only: when an exception's __str__ raises, format_exception_only swallows it and renders '<exception str() failed>' on a single line"""
import traceback


class _BadStr(Exception):
    def __str__(self):
        1 / 0


_err = traceback.format_exception_only(_BadStr, _BadStr())
assert len(_err) == 1, f"bad-str lines = {len(_err)!r}"
assert "<exception str() failed>" in _err[0], f"bad-str render: {_err[0]!r}"

print("format_exception_only_swallows_broken_str OK")
