# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "surface"
# case = "urlretrieve_is_callable"
# subject = "urllib.request.urlretrieve"
# kind = "mechanical"
# xfail = "mamba dotted-import quirk: urllib.request.urlretrieve -> None (repo-memory project_mamba_dotted_import_quirk)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""urllib.request.urlretrieve: urlretrieve_is_callable (surface)."""
import urllib.request

assert callable(urllib.request.urlretrieve)
print("urlretrieve_is_callable OK")
