# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "type"
# case = "Snapshot__compare_to__old_snapshot_as_Snapshot_wrong"
# subject = "tracemalloc.Snapshot.compare_to(old_snapshot: Snapshot)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tracemalloc.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tracemalloc.Snapshot.compare_to(old_snapshot: Snapshot); call it with the wrong type.

typeshed contract: old_snapshot is Snapshot. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from tracemalloc import Snapshot
obj = object.__new__(Snapshot)
try:
    obj.compare_to(_W(), "")  # old_snapshot: Snapshot <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
