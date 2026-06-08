# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "surface"
# case = "api_feedparser_is_present"
# subject = "email.feedparser"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""email.feedparser: api_feedparser_is_present (surface)."""
import email.feedparser

assert hasattr(email, "feedparser")
print("api_feedparser_is_present OK")
