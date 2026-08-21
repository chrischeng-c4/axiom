# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "498"
# dimension = "behavior"
# case = "inner_quotes_differ_from_delimiter"
# subject = "fstring.quoting"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.quoting: a field's string literal may use a different quote style, including inside a triple-quoted f-string: f'''{d["'"]}''' is 'squote' and f'''{d['"']}''' is 'dquote'"""
# an f-string field may contain differently-quoted string literals

d = {'"': "dquote", "'": "squote", "foo": "bar"}
assert f"""{d["'"]}""" == "squote"
assert f"""{d['"']}""" == "dquote"
assert f"{d['foo']}" == "bar"

print("inner_quotes_differ_from_delimiter OK")
