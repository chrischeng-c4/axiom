# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "type"
# case = "Snapshot__init__traces_as_Sequence_wrong"
# subject = "tracemalloc.Snapshot.__init__(traces: Sequence)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed traces"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tracemalloc.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed traces
# mamba-strict-type: TypeError
"""Type wall: tracemalloc.Snapshot.__init__(traces: Sequence); call it with the wrong type.

typeshed contract: traces is Sequence. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from tracemalloc import Snapshot
try:
    Snapshot(_W(), 0)  # traces: Sequence <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
