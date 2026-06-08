# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "positional_only"
# dimension = "behavior"
# case = "regular_param_positional_or_keyword"
# subject = "/"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""/: a regular (non-pos-only, non-kw-only) parameter can be passed positionally, by keyword, or mixed, all yielding the same result"""

# Rule: parameters with no `/` before them and no `*` are ordinary
# positional-or-keyword params — every call style yields the same result.
def _regular(x: int, y: int) -> int:
    return x + y

assert _regular(3, 4) == 7, "positional"
assert _regular(x=3, y=4) == 7, "keyword"
assert _regular(3, y=4) == 7, "mixed"

print("regular_param_positional_or_keyword OK")
