# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_pickle"
# dimension = "type"
# case = "load__file_as__ReadableFileobj_wrong"
# subject = "_pickle.load(file: _ReadableFileobj)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_pickle.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _pickle.load(file: _ReadableFileobj); call it with the wrong type.

typeshed contract: file is _ReadableFileobj. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _pickle import load
try:
    load(_W())  # file: _ReadableFileobj <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
