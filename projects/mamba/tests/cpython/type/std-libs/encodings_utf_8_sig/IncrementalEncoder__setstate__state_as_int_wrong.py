# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "encodings_utf_8_sig"
# dimension = "type"
# case = "IncrementalEncoder__setstate__state_as_int_wrong"
# subject = "encodings.utf_8_sig.IncrementalEncoder.setstate(state: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/encodings/utf_8_sig.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: encodings.utf_8_sig.IncrementalEncoder.setstate(state: int); call it with the wrong type.

typeshed contract: state is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from encodings.utf_8_sig import IncrementalEncoder
obj = object.__new__(IncrementalEncoder)
try:
    obj.setstate("not_an_int")  # state: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
