# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "behavior"
# case = "setlocale_query_returns_str"
# subject = "locale.setlocale"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_locale.py"
# status = "filled"
# ///
"""locale.setlocale: setlocale(category) with no locale arg queries (does not change) and returns a str for every standard LC_ category"""
import locale

# setlocale with only a category queries (does not change) the current
# value, returning a string for every standard LC_ category.
for cat in (
    locale.LC_ALL,
    locale.LC_CTYPE,
    locale.LC_TIME,
    locale.LC_COLLATE,
    locale.LC_MONETARY,
    locale.LC_NUMERIC,
):
    current = locale.setlocale(cat)
    assert isinstance(current, str), "setlocale query -> str"

print("setlocale_query_returns_str OK")
