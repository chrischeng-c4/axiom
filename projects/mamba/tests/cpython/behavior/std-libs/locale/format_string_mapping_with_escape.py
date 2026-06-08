# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "behavior"
# case = "format_string_mapping_with_escape"
# subject = "locale.format_string"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_locale.py"
# status = "filled"
# ///
"""locale.format_string: format_string with a mapping and an escaped %% literal matches plain %-formatting"""
import locale

# Mapping with an escaped %% literal.
assert (
    locale.format_string("%(foo)s %%d", {"foo": "bar"})
    == "%(foo)s %%d" % {"foo": "bar"}
), "format_string mapping with percent escape"

print("format_string_mapping_with_escape OK")
