# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "surface"
# case = "urlparse_is_callable"
# subject = "urllib.parse.urlparse"
# kind = "mechanical"
# xfail = "mamba dotted-import quirk: urllib.parse.urlparse -> None (repo-memory project_mamba_dotted_import_quirk)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""urllib.parse.urlparse: urlparse_is_callable (surface)."""
import urllib.parse

assert callable(urllib.parse.urlparse)
print("urlparse_is_callable OK")
