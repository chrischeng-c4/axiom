# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mmap"
# dimension = "type"
# case = "mmap____release_buffer____buffer_as_memoryview_wrong"
# subject = "mmap.mmap.__release_buffer__(buffer: memoryview)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed buffer"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mmap.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed buffer
# mamba-strict-type: TypeError
"""Type wall: mmap.mmap.__release_buffer__(buffer: memoryview); call it with the wrong type.

typeshed contract: buffer is memoryview. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from mmap import mmap
obj = object.__new__(mmap)
try:
    obj.__release_buffer__(12345)  # buffer: memoryview <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
