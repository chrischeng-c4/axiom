# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "comparison_returns_bool"
# subject = "operator.eq"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.eq: the comparison functions eq/ne/lt/le/gt/ge each return the bool singleton (is True / is False) matching the corresponding operator"""
import operator

_cases = [
    (operator.eq, 5, 5, True),
    (operator.ne, 5, 6, True),
    (operator.lt, 3, 5, True),
    (operator.le, 5, 5, True),
    (operator.gt, 7, 3, True),
    (operator.ge, 5, 4, True),
    (operator.eq, 5, 6, False),
    (operator.lt, 5, 3, False),
]
for _op, _a, _b, _expected in _cases:
    _result = _op(_a, _b)
    assert _result is _expected, f"{_op.__name__}({_a},{_b}) = {_result!r}"

print("comparison_returns_bool OK")
