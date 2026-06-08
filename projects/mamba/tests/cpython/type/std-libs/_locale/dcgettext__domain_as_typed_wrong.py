# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_locale"
# dimension = "type"
# case = "dcgettext__domain_as_typed_wrong"
# subject = "_locale.dcgettext(domain: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_locale.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _locale.dcgettext(domain: typed); call it with the wrong type.

typeshed contract: domain is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _locale import dcgettext
try:
    dcgettext(_W(), "", 0)  # domain: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
