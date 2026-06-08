# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "surface"
# case = "api_template_is_present"
# subject = "string.Template"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""string.Template: api_template_is_present (surface)."""
import string

assert hasattr(string, "Template")
print("api_template_is_present OK")
