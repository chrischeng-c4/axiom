# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "concurrent_interpreters"
# dimension = "type"
# case = "ExecutionFailed__init__excinfo_as_SimpleNamespace_wrong"
# subject = "concurrent.interpreters.ExecutionFailed.__init__(excinfo: SimpleNamespace)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/concurrent/interpreters.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: concurrent.interpreters.ExecutionFailed.__init__(excinfo: SimpleNamespace); call it with the wrong type.

typeshed contract: excinfo is SimpleNamespace. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from concurrent.interpreters import ExecutionFailed
try:
    ExecutionFailed(_W())  # excinfo: SimpleNamespace <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
