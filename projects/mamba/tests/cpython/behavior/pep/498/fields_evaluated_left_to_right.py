# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "498"
# dimension = "behavior"
# case = "fields_evaluated_left_to_right"
# subject = "fstring.evaluation"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.evaluation: replacement fields evaluate left to right: a Counter whose __format__ returns an incrementing count gives f'{c} {c}' == '1 2'"""
# field evaluation order is left to right, observable via side effects

class Counter:
    def __init__(self):
        self.i = 0
    def __format__(self, spec):
        self.i += 1
        return str(self.i)

c = Counter()
assert f"{c} {c}" == "1 2"

print("fields_evaluated_left_to_right OK")
