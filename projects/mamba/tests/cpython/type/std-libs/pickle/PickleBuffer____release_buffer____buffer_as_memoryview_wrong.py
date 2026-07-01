# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "type"
# case = "PickleBuffer____release_buffer____buffer_as_memoryview_wrong"
# subject = "pickle.PickleBuffer.__release_buffer__(buffer: memoryview)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pickle.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pickle.PickleBuffer.__release_buffer__(buffer: memoryview); call it with the wrong type.

typeshed contract: buffer is memoryview. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from pickle import PickleBuffer
obj = object.__new__(PickleBuffer)
try:
    obj.__release_buffer__(12345)  # buffer: memoryview <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
