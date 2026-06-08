# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "498"
# dimension = "behavior"
# case = "trailing_double_brace_is_literal_after_format"
# subject = "fstring.format_spec"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.format_spec: the spec is read greedily to the closing brace and a trailing }} is a literal brace: f'{1:}}}' is '1}' and f'{1:>3{5}}}}' right-aligns 1 in width 5 then appends '}'"""
# }} after a field's closing brace is a literal '}'

# The format spec is matched greedily up to the closing brace; the
# trailing }} is a literal brace appended after formatting.
assert f"{1:}}}" == "1}"
assert f"{1:>3{5}}}}" == ("                                  1" + "}")

print("trailing_double_brace_is_literal_after_format OK")
