# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "method_resolution"
# dimension = "type"
# case = "method_self_int_called_with_str"
# subject = "unbound method receiver contract"
# kind = "semantic"
# xfail = "unbound method receiver contract pending; currently MAMBA_TYPE_LEAKED"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
# mamba-xfail: unbound method receiver contract pending; currently MAMBA_TYPE_LEAKED
# mamba-strict-type: TypeError
"""Mamba runtime-type enforcement: method receiver type mismatch.

CPython 3.12: unbound-method-style call with wrong receiver type
goes through (CPython doesn't enforce `self` annotations).
Mamba: raises TypeError on the receiver-type contract.
"""


class Box:
    def get(self, which: int) -> int:
        return which * 2


try:
    # Call the unbound function with a non-Box self.
    result = Box.get("not_a_box", 3)  # type: ignore[arg-type]
    print("no_typeerror:", repr(result))
except TypeError as e:
    print("typeerror:", type(e).__name__, str(e)[:60])
