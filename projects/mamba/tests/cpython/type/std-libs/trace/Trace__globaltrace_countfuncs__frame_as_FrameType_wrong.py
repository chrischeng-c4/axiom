# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "trace"
# dimension = "type"
# case = "Trace__globaltrace_countfuncs__frame_as_FrameType_wrong"
# subject = "trace.Trace.globaltrace_countfuncs(frame: FrameType)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/trace.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: trace.Trace.globaltrace_countfuncs(frame: FrameType); call it with the wrong type.

typeshed contract: frame is FrameType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from trace import Trace
obj = object.__new__(Trace)
try:
    obj.globaltrace_countfuncs(_W(), "", None)  # frame: FrameType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
