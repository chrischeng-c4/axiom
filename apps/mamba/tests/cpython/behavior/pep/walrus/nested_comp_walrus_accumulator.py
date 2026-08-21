# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "walrus"
# dimension = "behavior"
# case = "nested_comp_walrus_accumulator"
# subject = ":="
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
""":=: a walrus that reads and writes the same enclosing name accumulates a running total across a comprehension: [(o := o + v) for v in range(5)] yields [0,1,3,6,10] and o == 10"""
# A walrus reading + writing the same enclosing name accumulates a running total.
outer2 = 0
inner_list = [(outer2 := outer2 + v) for v in range(5)]
# accumulates: 0, 0+1=1, 1+2=3, 3+3=6, 6+4=10
assert inner_list == [0, 1, 3, 6, 10], f"accumulate = {inner_list!r}"
assert outer2 == 10, f"outer2 = {outer2!r}"

print("nested_comp_walrus_accumulator OK")
