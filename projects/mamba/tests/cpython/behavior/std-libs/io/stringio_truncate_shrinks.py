# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "behavior"
# case = "stringio_truncate_shrinks"
# subject = "io.StringIO"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""io.StringIO: truncate() at the current position drops everything after it"""
import io

_trunc = io.StringIO("hello world")
_trunc.seek(5)
_trunc.truncate()
_trunc.seek(0)
assert _trunc.read() == "hello", "truncate drops everything after position"

print("stringio_truncate_shrinks OK")
