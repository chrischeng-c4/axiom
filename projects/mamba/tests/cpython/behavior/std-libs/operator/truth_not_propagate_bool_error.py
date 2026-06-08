# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "truth_not_propagate_bool_error"
# subject = "operator.truth"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.truth: truth and not_ delegate to __bool__; an exception raised inside __bool__ propagates unchanged out of both functions"""
import operator

class BoolRaises:
    def __bool__(self):
        raise SyntaxError("boom")


for _func in (operator.truth, operator.not_):
    _raised = False
    try:
        _func(BoolRaises())
    except SyntaxError:
        _raised = True
    assert _raised, f"{_func.__name__} should propagate __bool__ error"

print("truth_not_propagate_bool_error OK")
