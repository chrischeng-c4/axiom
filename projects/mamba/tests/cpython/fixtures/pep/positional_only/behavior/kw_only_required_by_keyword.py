# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "positional_only"
# dimension = "behavior"
# case = "kw_only_required_by_keyword"
# subject = "*"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""*: a keyword-only parameter (after `*`) is bound when passed by keyword: def f(*, n, m): return n * m; f(n=3, m=4) == 12"""

# Rule: parameters after a bare `*` are keyword-only and bind when supplied by
# keyword.
def _kw_only(*, n: int, m: int) -> int:
    return n * m

assert _kw_only(n=3, m=4) == 12, _kw_only(n=3, m=4)

print("kw_only_required_by_keyword OK")
