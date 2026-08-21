# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "operator_dispatch"
# dimension = "type"
# case = "list_minus_list_via_eval"
# subject = "operator dispatch TypeError"
# kind = "semantic"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Mamba strong-typing contract: list - list via eval().

list has no __sub__; both runtimes should raise TypeError. The
fixture prints `typeerror:` when the runtime honored its typing
contract; `no_typeerror:` when it let the operation through.
"""

try:
    result = eval("[1, 2] - [3]")
    print("no_typeerror:", repr(result))
except TypeError as e:
    print("typeerror:", type(e).__name__, str(e)[:60])
