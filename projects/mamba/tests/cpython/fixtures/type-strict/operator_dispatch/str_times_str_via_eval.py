# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "type-strict"
# lib = "operator_dispatch"
# dimension = "errors"
# case = "str_times_str_via_eval"
# subject = "operator dispatch TypeError"
# kind = "semantic"
# xfail = "eval operator TypeError dispatch pending; currently MAMBA_TYPE_LEAKED"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
# mamba-xfail: eval operator TypeError dispatch pending; currently MAMBA_TYPE_LEAKED
# mamba-strict-type: TypeError
"""Mamba strong-typing contract: str * str via eval().

Both runtimes should raise TypeError ("can't multiply sequence by
non-int of type 'str'"). The fixture prints `typeerror:` when the
runtime honored its typing contract; `no_typeerror:` when it let the
operation through and returned a value.
"""

try:
    result = eval("'a' * 'b'")
    print("no_typeerror:", repr(result))
except TypeError as e:
    print("typeerror:", type(e).__name__, str(e)[:60])
