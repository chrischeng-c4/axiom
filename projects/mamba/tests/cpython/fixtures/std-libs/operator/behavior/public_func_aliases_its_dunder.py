# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "public_func_aliases_its_dunder"
# subject = "operator"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator: each public function whose name has a dunder twin IS that twin object (operator.add is operator.__add__, etc.) across the whole module surface"""
import operator

# Each public function whose name has a dunder twin IS that twin.
# e.g. operator.add is operator.__add__.
_checked = 0
for _name in (n for n in dir(operator) if not n.startswith("_")):
    _dunder = getattr(operator, "__" + _name.strip("_") + "__", None)
    if _dunder is not None:
        assert _dunder is getattr(operator, _name), f"{_name} not aliased to its dunder"
        _checked += 1
assert _checked > 0, "expected at least one dunder-aliased function"

print("public_func_aliases_its_dunder OK")
