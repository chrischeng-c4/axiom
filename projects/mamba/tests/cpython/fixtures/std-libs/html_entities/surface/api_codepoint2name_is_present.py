# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "html_entities"
# dimension = "surface"
# case = "api_codepoint2name_is_present"
# subject = "html.entities.codepoint2name"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""html.entities.codepoint2name: api_codepoint2name_is_present (surface)."""
import html.entities

assert hasattr(html.entities, "codepoint2name")
print("api_codepoint2name_is_present OK")
