# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "type"
# case = "TracebackException__init__exc_type_as_type_wrong"
# subject = "traceback.TracebackException.__init__(exc_type: type)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed exc_type"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/traceback.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed exc_type
# mamba-strict-type: TypeError
"""Type wall: traceback.TracebackException.__init__(exc_type: type); call it with the wrong type.

typeshed contract: exc_type is type. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from traceback import TracebackException
try:
    TracebackException(_W(), None, None)  # exc_type: type <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
