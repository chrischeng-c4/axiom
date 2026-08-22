# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "behavior"
# case = "str_falls_back_to_repr"
# subject = "builtins.str"
# kind = "semantic"
# xfail = ""
# status = "filled"
# ///


class OnlyRepr:
    def __repr__(self):
        return "repr-sentinel"


assert str(OnlyRepr()) == "repr-sentinel"
