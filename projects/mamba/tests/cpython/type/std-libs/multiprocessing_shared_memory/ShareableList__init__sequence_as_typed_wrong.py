# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_shared_memory"
# dimension = "type"
# case = "ShareableList__init__sequence_as_typed_wrong"
# subject = "multiprocessing.shared_memory.ShareableList.__init__(sequence: typed)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed sequence"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/shared_memory.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed sequence
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.shared_memory.ShareableList.__init__(sequence: typed); call it with the wrong type.

typeshed contract: sequence is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from multiprocessing.shared_memory import ShareableList
try:
    ShareableList(_W())  # sequence: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
