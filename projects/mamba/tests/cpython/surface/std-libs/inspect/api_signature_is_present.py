# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "api_signature_is_present"
# subject = "inspect.Signature"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""inspect.Signature: api_signature_is_present (surface)."""
import inspect

assert hasattr(inspect, "Signature")
print("api_signature_is_present OK")
