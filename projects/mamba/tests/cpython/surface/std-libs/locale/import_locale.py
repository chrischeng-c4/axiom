# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "surface"
# case = "import_locale"
# subject = "locale"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""locale: import_locale (surface)."""
import locale

assert hasattr(locale, "setlocale")
print("import_locale OK")
