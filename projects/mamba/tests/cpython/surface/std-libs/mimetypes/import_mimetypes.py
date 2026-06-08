# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "surface"
# case = "import_mimetypes"
# subject = "mimetypes"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""mimetypes: import_mimetypes (surface)."""
import mimetypes

assert hasattr(mimetypes, "guess_type")
print("import_mimetypes OK")
