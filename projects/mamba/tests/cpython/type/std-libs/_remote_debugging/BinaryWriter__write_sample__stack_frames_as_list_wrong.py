# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_remote_debugging"
# dimension = "type"
# case = "BinaryWriter__write_sample__stack_frames_as_list_wrong"
# subject = "_remote_debugging.BinaryWriter.write_sample(stack_frames: list)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed stack_frames"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_remote_debugging.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed stack_frames
# mamba-strict-type: TypeError
"""Type wall: _remote_debugging.BinaryWriter.write_sample(stack_frames: list); call it with the wrong type.

typeshed contract: stack_frames is list. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _remote_debugging import BinaryWriter
obj = object.__new__(BinaryWriter)
try:
    obj.write_sample(12345, 0)  # stack_frames: list <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
