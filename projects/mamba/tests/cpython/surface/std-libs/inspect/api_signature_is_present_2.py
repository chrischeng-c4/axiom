# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "api_signature_is_present_2"
# subject = "inspect.signature"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""inspect.signature: api_signature_is_present_2 (surface)."""
import inspect

assert hasattr(inspect, "signature")
print("api_signature_is_present_2 OK")
