# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "surface"
# case = "api_iterators_is_present"
# subject = "email.iterators"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""email.iterators: api_iterators_is_present (surface)."""
import email.iterators

assert hasattr(email, "iterators")
print("api_iterators_is_present OK")
