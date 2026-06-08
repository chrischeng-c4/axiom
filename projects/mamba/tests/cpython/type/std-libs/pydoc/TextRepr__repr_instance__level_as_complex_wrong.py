# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pydoc"
# dimension = "type"
# case = "TextRepr__repr_instance__level_as_complex_wrong"
# subject = "pydoc.TextRepr.repr_instance(level: complex)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed level"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pydoc.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed level
# mamba-strict-type: TypeError
"""Type wall: pydoc.TextRepr.repr_instance(level: complex); call it with the wrong type.

typeshed contract: level is complex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from pydoc import TextRepr
obj = object.__new__(TextRepr)
try:
    obj.repr_instance(None, "not_a_complex")  # level: complex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
