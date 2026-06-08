# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "behavior"
# case = "cleandoc_dedents_uniform_indentation"
# subject = "inspect.cleandoc"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.cleandoc: cleandoc() strips uniform leading indentation and the leading blank line"""
import inspect

assert (
    inspect.cleandoc("An\n    indented\n    docstring.")
    == "An\nindented\ndocstring."
), "cleandoc dedent"

print("cleandoc_dedents_uniform_indentation OK")
