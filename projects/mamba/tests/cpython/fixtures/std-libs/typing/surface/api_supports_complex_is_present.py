# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "surface"
# case = "api_supports_complex_is_present"
# subject = "typing.SupportsComplex"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""typing.SupportsComplex: api_supports_complex_is_present (surface)."""
import typing

assert hasattr(typing, "SupportsComplex")
print("api_supports_complex_is_present OK")
