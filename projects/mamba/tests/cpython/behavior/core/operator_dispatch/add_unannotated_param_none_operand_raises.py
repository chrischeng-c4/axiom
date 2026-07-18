# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "operator_dispatch"
# dimension = "behavior"
# case = "add_unannotated_param_none_operand_raises"
# subject = "binary + operand type dispatch"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""Binary `+` type-mismatch check must hard-fault when a None operand
arrives through an unannotated (Any-typed) function parameter, matching
CPython and mamba's own local-variable behavior (#1938).

An unannotated parameter is statically Any, so mamba cannot narrow its
type at compile time the way it can for a direct `x = None` assignment;
the None-vs-str mismatch must be caught at runtime by the `+` dispatcher
instead.
"""


def combine(value):
    return value + "i"


try:
    result = combine(None)
    raise AssertionError(f"expected TypeError, got {result!r}")
except TypeError as e:
    assert str(e) == "unsupported operand type(s) for +: 'NoneType' and 'str'", str(e)

print("add_unannotated_param_none_operand_raises OK")
