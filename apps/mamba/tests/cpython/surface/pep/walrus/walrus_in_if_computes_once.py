# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "walrus"
# dimension = "surface"
# case = "walrus_in_if_computes_once"
# subject = ":="
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
""":=: a walrus in an if-condition computes a value once and the binding is reused in the chosen branch body"""
# := in an if-condition computes once; the binding is reused in the body.
def expensive(x: int) -> int:
    return x * x

computed = None
if (computed := expensive(7)) > 40:
    assert computed == 49, f"if walrus = {computed!r}"
else:
    raise AssertionError("should have branched to if")

print("walrus_in_if_computes_once OK")
