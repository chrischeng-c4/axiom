# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "surface"
# case = "api_register_is_present"
# subject = "codecs.register"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""codecs.register: api_register_is_present (surface)."""
import codecs

assert hasattr(codecs, "register")
print("api_register_is_present OK")
