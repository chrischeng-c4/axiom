# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tomllib"
# dimension = "surface"
# case = "api_toml_decode_error_is_present"
# subject = "tomllib.TOMLDecodeError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tomllib.TOMLDecodeError: api_toml_decode_error_is_present (surface)."""
import tomllib

assert hasattr(tomllib, "TOMLDecodeError")
print("api_toml_decode_error_is_present OK")
