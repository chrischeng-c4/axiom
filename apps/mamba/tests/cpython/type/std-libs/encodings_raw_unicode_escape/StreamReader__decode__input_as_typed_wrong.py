# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "encodings_raw_unicode_escape"
# dimension = "type"
# case = "StreamReader__decode__input_as_typed_wrong"
# subject = "encodings.raw_unicode_escape.StreamReader.decode(input: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/encodings/raw_unicode_escape.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: encodings.raw_unicode_escape.StreamReader.decode(input: typed); call it with the wrong type.

typeshed contract: input is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from encodings.raw_unicode_escape import StreamReader
obj = object.__new__(StreamReader)
try:
    obj.decode(_W())  # input: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
