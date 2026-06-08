# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "572"
# dimension = "behavior"
# case = "genexp_walrus_no_preleak_until_consumed"
# subject = ":="
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
""":=: a generator expression does NOT pre-leak its walrus target before consumption; the bound name only appears after the generator is iterated"""
# A generator expression does NOT pre-leak its walrus target before it is
# consumed; the name only appears after iteration.
seed = 1
genexp = ((c := i + seed) for i in [1, 2, 3, 4])
assert "c" not in locals()
produced = list(genexp)
assert produced == [2, 3, 4, 5]
assert c == 5

print("genexp_walrus_no_preleak_until_consumed OK")
