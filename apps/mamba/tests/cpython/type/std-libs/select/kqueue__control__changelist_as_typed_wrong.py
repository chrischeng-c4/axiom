# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "select"
# dimension = "type"
# case = "kqueue__control__changelist_as_typed_wrong"
# subject = "select.kqueue.control(changelist: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/select.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: select.kqueue.control(changelist: typed); call it with the wrong type.

typeshed contract: changelist is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from select import kqueue
obj = object.__new__(kqueue)
try:
    obj.control(_W(), 0)  # changelist: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
