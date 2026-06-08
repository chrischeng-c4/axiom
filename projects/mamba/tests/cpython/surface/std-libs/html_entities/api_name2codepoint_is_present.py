# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "html_entities"
# dimension = "surface"
# case = "api_name2codepoint_is_present"
# subject = "html.entities.name2codepoint"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""html.entities.name2codepoint: api_name2codepoint_is_present (surface)."""
import html.entities

assert hasattr(html.entities, "name2codepoint")
print("api_name2codepoint_is_present OK")
