# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pdb"
# dimension = "type"
# case = "Pdb__sigint_handler__signum_as_Signals_wrong"
# subject = "pdb.Pdb.sigint_handler(signum: Signals)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pdb.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pdb.Pdb.sigint_handler(signum: Signals); call it with the wrong type.

typeshed contract: signum is Signals. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from pdb import Pdb
obj = object.__new__(Pdb)
try:
    obj.sigint_handler(_W(), None)  # signum: Signals <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
