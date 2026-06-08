# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_heapq"
# dimension = "type"
# case = "heapify_max__heap_as_list_wrong"
# subject = "_heapq.heapify_max(heap: list)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed heap"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_heapq.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed heap
# mamba-strict-type: TypeError
"""Type wall: _heapq.heapify_max(heap: list); call it with the wrong type.

typeshed contract: heap is list. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _heapq import heapify_max
try:
    heapify_max(12345)  # heap: list <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
