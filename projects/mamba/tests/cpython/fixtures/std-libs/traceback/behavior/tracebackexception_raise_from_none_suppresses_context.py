# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "behavior"
# case = "tracebackexception_raise_from_none_suppresses_context"
# subject = "traceback.TracebackException"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""traceback.TracebackException: 'raise RuntimeError(...) from None' inside an except sets __suppress_context__ True on the from_exception capture, and chained format shows 'RuntimeError: chained'"""
import traceback

try:
    try:
        raise ValueError("orig")
    except ValueError:
        raise RuntimeError("chained") from None
except RuntimeError as e:
    suppressed = traceback.TracebackException.from_exception(e)
    _formatted = "".join(suppressed.format())
assert suppressed.__suppress_context__ is True, \
    f"from-None suppress = {suppressed.__suppress_context__!r}"
assert "RuntimeError: chained" in _formatted, f"chained: {_formatted!r}"

print("tracebackexception_raise_from_none_suppresses_context OK")
