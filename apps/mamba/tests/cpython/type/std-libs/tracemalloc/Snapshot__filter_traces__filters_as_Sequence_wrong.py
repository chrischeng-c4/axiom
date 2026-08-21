# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "type"
# case = "Snapshot__filter_traces__filters_as_Sequence_wrong"
# subject = "tracemalloc.Snapshot.filter_traces(filters: Sequence)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed filters"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tracemalloc.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed filters
# mamba-strict-type: TypeError
"""Type wall: tracemalloc.Snapshot.filter_traces(filters: Sequence); call it with the wrong type.

typeshed contract: filters is Sequence. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from tracemalloc import Snapshot
obj = object.__new__(Snapshot)
try:
    obj.filter_traces(_W())  # filters: Sequence <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
