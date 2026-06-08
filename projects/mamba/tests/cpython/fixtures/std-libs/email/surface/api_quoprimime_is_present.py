# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "surface"
# case = "api_quoprimime_is_present"
# subject = "email.quoprimime"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""email.quoprimime: api_quoprimime_is_present (surface)."""
import email.quoprimime

assert hasattr(email, "quoprimime")
print("api_quoprimime_is_present OK")
