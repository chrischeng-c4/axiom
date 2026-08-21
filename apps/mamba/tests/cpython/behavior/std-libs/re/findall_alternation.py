# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "findall_alternation"
# subject = "re.findall"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""re.findall: findall over an alternation collects each alternative hit: r'cat|dog' on 'I have a cat and a dog' is ['cat','dog']"""
import re

assert re.findall(r"cat|dog", "I have a cat and a dog") == ["cat", "dog"]

print("findall_alternation OK")
