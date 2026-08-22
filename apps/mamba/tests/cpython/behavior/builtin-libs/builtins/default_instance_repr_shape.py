# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "behavior"
# case = "default_instance_repr_shape"
# subject = "builtins.repr"
# kind = "semantic"
# xfail = ""
# status = "filled"
# ///

import re


class Plain:
    pass


rendered = repr(Plain())
assert re.sub(r"0x[0-9a-fA-F]+", "0x...", rendered) == "<__main__.Plain object at 0x...>"
