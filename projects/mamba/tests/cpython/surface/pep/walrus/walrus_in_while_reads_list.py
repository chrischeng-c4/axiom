# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "walrus"
# dimension = "surface"
# case = "walrus_in_while_reads_list"
# subject = ":="
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
""":=: a walrus in a while-condition captures each list element so the loop body can append the captured value, stopping when the guard fails"""
# := in a while-condition: capture each element, stop when the guard fails.
items = [3, 1, 4, 1, 5, 9]
result = []
idx = 0
while idx < len(items) and (v := items[idx]) < 9:
    result.append(v)
    idx += 1
assert result == [3, 1, 4, 1, 5], f"while result = {result!r}"

print("walrus_in_while_reads_list OK")
