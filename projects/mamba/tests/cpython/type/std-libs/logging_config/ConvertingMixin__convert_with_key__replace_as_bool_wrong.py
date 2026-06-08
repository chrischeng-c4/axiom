# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_config"
# dimension = "type"
# case = "ConvertingMixin__convert_with_key__replace_as_bool_wrong"
# subject = "logging.config.ConvertingMixin.convert_with_key(replace: bool)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed replace"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/logging/config.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed replace
# mamba-strict-type: TypeError
"""Type wall: logging.config.ConvertingMixin.convert_with_key(replace: bool); call it with the wrong type.

typeshed contract: replace is bool. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from logging.config import ConvertingMixin
obj = object.__new__(ConvertingMixin)
try:
    obj.convert_with_key(None, None, "not_a_bool")  # replace: bool <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
