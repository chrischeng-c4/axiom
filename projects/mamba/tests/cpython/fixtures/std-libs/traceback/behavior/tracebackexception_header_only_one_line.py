# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "behavior"
# case = "tracebackexception_header_only_one_line"
# subject = "traceback.TracebackException"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""traceback.TracebackException: a header-only TracebackException(Exception, Exception('haven'), None) with no traceback formats to the single line 'Exception: haven\\n'"""
import traceback

header = traceback.TracebackException(Exception, Exception("haven"), None)
assert list(header.format()) == ["Exception: haven\n"], \
    f"header format = {list(header.format())!r}"

print("tracebackexception_header_only_one_line OK")
