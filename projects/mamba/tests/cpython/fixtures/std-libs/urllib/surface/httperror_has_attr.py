# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "surface"
# case = "httperror_has_attr"
# subject = "urllib.error.HTTPError"
# kind = "mechanical"
# xfail = "mamba dotted-import quirk: urllib.error.HTTPError -> None (repo-memory project_mamba_dotted_import_quirk)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""urllib.error.HTTPError: httperror_has_attr (surface)."""
import urllib.error

assert hasattr(urllib.error.HTTPError, "__cause__")
print("httperror_has_attr OK")
