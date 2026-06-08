# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "surface"
# case = "api_generator_is_present"
# subject = "email.generator"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""email.generator: api_generator_is_present (surface)."""
import email.generator

assert hasattr(email, "generator")
print("api_generator_is_present OK")
