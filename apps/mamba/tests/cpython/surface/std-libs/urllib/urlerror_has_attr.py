# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "surface"
# case = "urlerror_has_attr"
# subject = "urllib.error.URLError"
# kind = "mechanical"
# xfail = "mamba dotted-import quirk: urllib.error.URLError -> None (repo-memory project_mamba_dotted_import_quirk)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""urllib.error.URLError: urlerror_has_attr (surface)."""
import urllib.error

assert hasattr(urllib.error.URLError, "__cause__")
print("urlerror_has_attr OK")
