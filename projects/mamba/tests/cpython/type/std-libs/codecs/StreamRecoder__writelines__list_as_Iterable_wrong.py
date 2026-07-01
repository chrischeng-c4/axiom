# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "type"
# case = "StreamRecoder__writelines__list_as_Iterable_wrong"
# subject = "codecs.StreamRecoder.writelines(list: Iterable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/codecs.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: codecs.StreamRecoder.writelines(list: Iterable); call it with the wrong type.

typeshed contract: list is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from codecs import StreamRecoder
obj = object.__new__(StreamRecoder)
try:
    obj.writelines(_W())  # list: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
