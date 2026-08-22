# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "percent_format_arity"
# dimension = "behavior"
# case = "percent_format_positional_dict_is_scalar"
# subject = "str.percent_format"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""str.percent_format: a positional percent s format treats a one-key dict as one scalar."""
# formatting syntax uses only Python built-ins

assert "%s" % {"name": "Ada"} == "{'name': 'Ada'}"

print("percent_format_positional_dict_is_scalar OK")
