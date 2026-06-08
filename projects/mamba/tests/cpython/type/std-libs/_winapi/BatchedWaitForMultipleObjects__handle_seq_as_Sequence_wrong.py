# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_winapi"
# dimension = "type"
# case = "BatchedWaitForMultipleObjects__handle_seq_as_Sequence_wrong"
# subject = "_winapi.BatchedWaitForMultipleObjects(handle_seq: Sequence)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed handle_seq"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_winapi.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed handle_seq
# mamba-strict-type: TypeError
"""Type wall: _winapi.BatchedWaitForMultipleObjects(handle_seq: Sequence); call it with the wrong type.

typeshed contract: handle_seq is Sequence. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _winapi import BatchedWaitForMultipleObjects
try:
    BatchedWaitForMultipleObjects(_W(), True)  # handle_seq: Sequence <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
