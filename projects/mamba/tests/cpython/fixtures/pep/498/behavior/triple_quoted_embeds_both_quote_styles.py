# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "498"
# dimension = "behavior"
# case = "triple_quoted_embeds_both_quote_styles"
# subject = "fstring.quoting"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.quoting: a triple-quoted f-string lets a field embed both quote styles: f'''{"eric's"}''' is "eric's" and f'''{'xeric"sy'}''' is 'xeric"sy'"""
# triple-quoted f-strings relax the inner quote constraint

assert f"""{"eric's"}""" == "eric's"
assert f"""{'xeric"sy'}""" == 'xeric"sy'

print("triple_quoted_embeds_both_quote_styles OK")
