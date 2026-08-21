# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "type"
# case = "Match__group__group_as_Literal_wrong"
# subject = "re.Match.group(group: Literal)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed group"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/re.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed group
# mamba-strict-type: TypeError
"""Type wall: re.Match.group(group: Literal); call it with the wrong type.

typeshed contract: group is Literal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from re import Match
obj = object.__new__(Match)
try:
    obj.group(_W())  # group: Literal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
