# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "572"
# dimension = "behavior"
# case = "genexp_arg_walrus_leaks_after_consume"
# subject = ":="
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
""":=: a walrus inside a generator-expression argument to a builtin (any) leaks to the surrounding scope once the generator has been consumed"""
# A walrus inside a generator expression argument to a builtin also
# leaks once the generator has been consumed.
contains_one = any((last := num) == 1 for num in [3, 2, 1])
assert contains_one is True
assert last == 1

print("genexp_arg_walrus_leaks_after_consume OK")
