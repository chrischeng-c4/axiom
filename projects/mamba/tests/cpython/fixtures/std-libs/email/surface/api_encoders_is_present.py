# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "surface"
# case = "api_encoders_is_present"
# subject = "email.encoders"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""email.encoders: api_encoders_is_present (surface)."""
import email.encoders

assert hasattr(email, "encoders")
print("api_encoders_is_present OK")
