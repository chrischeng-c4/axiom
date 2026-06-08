# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "fstrings"
# dimension = "behavior"
# case = "field_evaluated_in_enclosing_scope"
# subject = "fstring.evaluation"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.evaluation: a field expression is evaluated in the enclosing scope with its side effects: a counter-incrementing call inside f'{next_val()}' yields '1' and bumps the module counter to 1"""
# replacement fields evaluate in the enclosing scope, side effects included

counter = 0


def next_val() -> int:
    global counter
    counter += 1
    return counter


s = f"{next_val()}"
assert s == "1", f"expr eval = {s!r}"
assert counter == 1, f"counter = {counter!r}"

print("field_evaluated_in_enclosing_scope OK")
