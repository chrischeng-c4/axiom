# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "encodings_rot_13"
# dimension = "type"
# case = "rot13__infile_as_SupportsRead_wrong"
# subject = "encodings.rot_13.rot13(infile: SupportsRead)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/encodings/rot_13.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: encodings.rot_13.rot13(infile: SupportsRead); call it with the wrong type.

typeshed contract: infile is SupportsRead. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from encodings.rot_13 import rot13
try:
    rot13(_W(), None)  # infile: SupportsRead <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
