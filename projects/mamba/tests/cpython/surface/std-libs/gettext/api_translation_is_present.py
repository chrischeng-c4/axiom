# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gettext"
# dimension = "surface"
# case = "api_translation_is_present"
# subject = "gettext.translation"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""gettext.translation: api_translation_is_present (surface)."""
import gettext

assert hasattr(gettext, "translation")
print("api_translation_is_present OK")
