# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "abc"
# dimension = "type"
# case = "abstractclassmethod__init__callable_as_Callable_wrong"
# subject = "abc.abstractclassmethod.__init__(callable: Callable)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed callable"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/abc.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed callable
# mamba-strict-type: TypeError
"""Type wall: abc.abstractclassmethod.__init__(callable: Callable); call it with the wrong type.

typeshed contract: callable is Callable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from abc import abstractclassmethod
try:
    abstractclassmethod(_W())  # callable: Callable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
