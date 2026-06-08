# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "surface"
# case = "api_xmlcharrefreplace_errors_is_present"
# subject = "codecs.xmlcharrefreplace_errors"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""codecs.xmlcharrefreplace_errors: api_xmlcharrefreplace_errors_is_present (surface)."""
import codecs

assert hasattr(codecs, "xmlcharrefreplace_errors")
print("api_xmlcharrefreplace_errors_is_present OK")
