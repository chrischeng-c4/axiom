# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "surface"
# case = "contenttooshorterror_has_attr"
# subject = "urllib.error.ContentTooShortError"
# kind = "mechanical"
# xfail = "mamba dotted-import quirk: urllib.error.ContentTooShortError -> None (repo-memory project_mamba_dotted_import_quirk)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""urllib.error.ContentTooShortError: contenttooshorterror_has_attr (surface)."""
import urllib.error

assert hasattr(urllib.error.ContentTooShortError, "__cause__")
print("contenttooshorterror_has_attr OK")
