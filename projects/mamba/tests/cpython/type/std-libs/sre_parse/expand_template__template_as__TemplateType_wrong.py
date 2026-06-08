# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sre_parse"
# dimension = "type"
# case = "expand_template__template_as__TemplateType_wrong"
# subject = "sre_parse.expand_template(template: _TemplateType)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/sre_parse.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: sre_parse.expand_template(template: _TemplateType); call it with the wrong type.

typeshed contract: template is _TemplateType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from sre_parse import expand_template
try:
    expand_template(_W(), None)  # template: _TemplateType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
