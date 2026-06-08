# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "behavior"
# case = "format_string_mapping_passthrough"
# subject = "locale.format_string"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_locale.py"
# status = "filled"
# ///
"""locale.format_string: format_string with a %(name)s mapping passes straight through to plain %-formatting"""
import locale

# A %(name)s mapping passes straight through to %-formatting. Without
# grouping the output matches plain %-formatting, keeping these
# assertions locale-independent.
assert (
    locale.format_string("%(foo)s bing.", {"foo": "bar"})
    == "%(foo)s bing." % {"foo": "bar"}
), "format_string mapping with trailing text"
assert (
    locale.format_string("%(foo)s", {"foo": "bar"})
    == "%(foo)s" % {"foo": "bar"}
), "format_string bare mapping"

print("format_string_mapping_passthrough OK")
