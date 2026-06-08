# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "498"
# dimension = "behavior"
# case = "loop_variable_resolves_freshly"
# subject = "fstring.scope"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.scope: a loop variable resolves freshly each iteration: f'i:{i}' over range(5) yields ['i:0','i:1','i:2','i:3','i:4']"""
# a field re-reads the loop variable on each pass

seen = []
for i in range(5):
    seen.append(f"i:{i}")
assert seen == ["i:0", "i:1", "i:2", "i:3", "i:4"]

print("loop_variable_resolves_freshly OK")
