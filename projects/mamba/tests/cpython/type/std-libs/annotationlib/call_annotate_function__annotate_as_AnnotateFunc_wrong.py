# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "annotationlib"
# dimension = "type"
# case = "call_annotate_function__annotate_as_AnnotateFunc_wrong"
# subject = "annotationlib.call_annotate_function(annotate: AnnotateFunc)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed annotate"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/annotationlib.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed annotate
# mamba-strict-type: TypeError
"""Type wall: annotationlib.call_annotate_function(annotate: AnnotateFunc); call it with the wrong type.

typeshed contract: annotate is AnnotateFunc. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from annotationlib import call_annotate_function
try:
    call_annotate_function(_W(), None)  # annotate: AnnotateFunc <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
