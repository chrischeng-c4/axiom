# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gettext"
# dimension = "surface"
# case = "api_null_translations_is_present"
# subject = "gettext.NullTranslations"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""gettext.NullTranslations: api_null_translations_is_present (surface)."""
import gettext

assert hasattr(gettext, "NullTranslations")
print("api_null_translations_is_present OK")
