# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "fstrings"
# dimension = "surface"
# case = "repr_builtin_is_callable"
# subject = "repr"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""repr: repr_builtin_is_callable (surface)."""
# the !r conversion flag dispatches via the repr() builtin

assert callable(repr)
print("repr_builtin_is_callable OK")
