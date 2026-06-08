# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_spawn"
# dimension = "type"
# case = "is_forking__argv_as_Sequence_wrong"
# subject = "multiprocessing.spawn.is_forking(argv: Sequence)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed argv"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/spawn.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed argv
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.spawn.is_forking(argv: Sequence); call it with the wrong type.

typeshed contract: argv is Sequence. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from multiprocessing.spawn import is_forking
try:
    is_forking(_W())  # argv: Sequence <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
