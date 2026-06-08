# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "surface"
# case = "api_warnings_is_present"
# subject = "ssl.warnings"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ssl.warnings: api_warnings_is_present (surface)."""
import ssl

assert hasattr(ssl, "warnings")
print("api_warnings_is_present OK")
