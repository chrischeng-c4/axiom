# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uu"
# dimension = "type"
# case = "encode__in_file_as__File_wrong"
# subject = "uu.encode(in_file: _File)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/uu.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: uu.encode(in_file: _File); call it with the wrong type.

typeshed contract: in_file is _File. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from uu import encode
try:
    encode(_W(), None)  # in_file: _File <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
