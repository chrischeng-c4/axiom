# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "html_entities"
# dimension = "surface"
# case = "api_entitydefs_is_present"
# subject = "html.entities.entitydefs"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""html.entities.entitydefs: api_entitydefs_is_present (surface)."""
import html.entities

assert hasattr(html.entities, "entitydefs")
print("api_entitydefs_is_present OK")
