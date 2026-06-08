# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "surface"
# case = "api_create_default_context_is_present"
# subject = "ssl.create_default_context"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ssl.create_default_context: api_create_default_context_is_present (surface)."""
import ssl

assert hasattr(ssl, "create_default_context")
print("api_create_default_context_is_present OK")
