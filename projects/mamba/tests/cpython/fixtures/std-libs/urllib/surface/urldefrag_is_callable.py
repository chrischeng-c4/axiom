# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "surface"
# case = "urldefrag_is_callable"
# subject = "urllib.parse.urldefrag"
# kind = "mechanical"
# xfail = "mamba dotted-import quirk: urllib.parse.urldefrag -> None (repo-memory project_mamba_dotted_import_quirk)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""urllib.parse.urldefrag: urldefrag_is_callable (surface)."""
import urllib.parse

assert callable(urllib.parse.urldefrag)
print("urldefrag_is_callable OK")
