# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "html_entities"
# dimension = "surface"
# case = "api_html5_is_present"
# subject = "html.entities.html5"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""html.entities.html5: api_html5_is_present (surface)."""
import html.entities

assert hasattr(html.entities, "html5")
print("api_html5_is_present OK")
