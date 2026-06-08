# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "572"
# dimension = "behavior"
# case = "comp_running_total_accumulator"
# subject = ":="
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
""":=: a walrus that reads and writes the same enclosing name accumulates a running total across a comprehension: [(total := total + v) for v in range(5)] yields [0,1,3,6,10] and total == 10"""
# A walrus that reads and writes the same enclosing name accumulates a
# running total across the comprehension.
total = 0
partial = [(total := total + v) for v in range(5)]
assert partial == [0, 1, 3, 6, 10]
assert total == 10

print("comp_running_total_accumulator OK")
