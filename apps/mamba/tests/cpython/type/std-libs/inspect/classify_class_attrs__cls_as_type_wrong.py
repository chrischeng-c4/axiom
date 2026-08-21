# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "type"
# case = "classify_class_attrs__cls_as_type_wrong"
# subject = "inspect.classify_class_attrs(cls: type)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/inspect.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: inspect.classify_class_attrs(cls: type); call it with the wrong type.

typeshed contract: cls is type. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from inspect import classify_class_attrs
try:
    classify_class_attrs(_W())  # cls: type <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
