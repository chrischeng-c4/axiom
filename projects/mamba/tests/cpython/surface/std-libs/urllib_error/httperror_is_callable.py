# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_error"
# dimension = "surface"
# case = "httperror_is_callable"
# subject = "urllib.error.HTTPError"
# kind = "mechanical"
# xfail = "mamba dotted-import quirk: import urllib.error; urllib.error.HTTPError -> None (repo-memory project_mamba_dotted_import_quirk)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""urllib.error.HTTPError: httperror_is_callable (surface)."""
import urllib.error

assert callable(urllib.error.HTTPError)
print("httperror_is_callable OK")
