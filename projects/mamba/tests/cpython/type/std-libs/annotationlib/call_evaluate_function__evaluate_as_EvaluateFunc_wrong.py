# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "annotationlib"
# dimension = "type"
# case = "call_evaluate_function__evaluate_as_EvaluateFunc_wrong"
# subject = "annotationlib.call_evaluate_function(evaluate: EvaluateFunc)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed evaluate"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/annotationlib.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed evaluate
# mamba-strict-type: TypeError
"""Type wall: annotationlib.call_evaluate_function(evaluate: EvaluateFunc); call it with the wrong type.

typeshed contract: evaluate is EvaluateFunc. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from annotationlib import call_evaluate_function
try:
    call_evaluate_function(_W(), None)  # evaluate: EvaluateFunc <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
