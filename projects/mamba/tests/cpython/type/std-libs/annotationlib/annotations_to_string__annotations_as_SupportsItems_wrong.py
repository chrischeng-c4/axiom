# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "annotationlib"
# dimension = "type"
# case = "annotations_to_string__annotations_as_SupportsItems_wrong"
# subject = "annotationlib.annotations_to_string(annotations: SupportsItems)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/annotationlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: annotationlib.annotations_to_string(annotations: SupportsItems); call it with the wrong type.

typeshed contract: annotations is SupportsItems. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from annotationlib import annotations_to_string
try:
    annotations_to_string(_W())  # annotations: SupportsItems <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
