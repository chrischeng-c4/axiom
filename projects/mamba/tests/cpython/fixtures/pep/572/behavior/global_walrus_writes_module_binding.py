# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "572"
# dimension = "behavior"
# case = "global_walrus_writes_module_binding"
# subject = ":="
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
""":=: a walrus inside a function declared global writes the module-level binding"""
# A walrus inside a function with `global` writes the module binding.
g = 1

def bump():
    global g
    (g := 20)

bump()
assert g == 20

print("global_walrus_writes_module_binding OK")
