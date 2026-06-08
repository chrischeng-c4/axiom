# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing_extensions"
# dimension = "type"
# case = "evaluate_forward_ref__forward_ref_as_ForwardRef_wrong"
# subject = "typing_extensions.evaluate_forward_ref(forward_ref: ForwardRef)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed forward_ref"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/typing_extensions.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed forward_ref
# mamba-strict-type: TypeError
"""Type wall: typing_extensions.evaluate_forward_ref(forward_ref: ForwardRef); call it with the wrong type.

typeshed contract: forward_ref is ForwardRef. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from typing_extensions import evaluate_forward_ref
try:
    evaluate_forward_ref(_W())  # forward_ref: ForwardRef <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
