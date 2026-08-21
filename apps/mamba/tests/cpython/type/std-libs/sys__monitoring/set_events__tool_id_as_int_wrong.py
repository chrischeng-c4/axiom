# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys__monitoring"
# dimension = "type"
# case = "set_events__tool_id_as_int_wrong"
# subject = "sys._monitoring.set_events(tool_id: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/sys/_monitoring.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: sys._monitoring.set_events(tool_id: int); call it with the wrong type.

typeshed contract: tool_id is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from sys._monitoring import set_events
try:
    set_events("not_an_int", 0)  # tool_id: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
