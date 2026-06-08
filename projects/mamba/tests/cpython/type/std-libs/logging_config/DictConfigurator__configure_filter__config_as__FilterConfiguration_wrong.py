# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_config"
# dimension = "type"
# case = "DictConfigurator__configure_filter__config_as__FilterConfiguration_wrong"
# subject = "logging.config.DictConfigurator.configure_filter(config: _FilterConfiguration)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/logging/config.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: logging.config.DictConfigurator.configure_filter(config: _FilterConfiguration); call it with the wrong type.

typeshed contract: config is _FilterConfiguration. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from logging.config import DictConfigurator
obj = object.__new__(DictConfigurator)
try:
    obj.configure_filter(_W())  # config: _FilterConfiguration <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
