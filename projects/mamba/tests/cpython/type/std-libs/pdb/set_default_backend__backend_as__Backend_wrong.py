# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pdb"
# dimension = "type"
# case = "set_default_backend__backend_as__Backend_wrong"
# subject = "pdb.set_default_backend(backend: _Backend)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pdb.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pdb.set_default_backend(backend: _Backend); call it with the wrong type.

typeshed contract: backend is _Backend. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from pdb import set_default_backend
try:
    set_default_backend(_W())  # backend: _Backend <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
