# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_runner"
# dimension = "type"
# case = "TextTestResult__init__stream_as__StreamT_wrong"
# subject = "unittest.runner.TextTestResult.__init__(stream: _StreamT)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed stream"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/runner.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed stream
# mamba-strict-type: TypeError
"""Type wall: unittest.runner.TextTestResult.__init__(stream: _StreamT); call it with the wrong type.

typeshed contract: stream is _StreamT. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.runner import TextTestResult
try:
    TextTestResult(_W(), True, 0)  # stream: _StreamT <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
