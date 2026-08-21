# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_config"
# dimension = "type"
# case = "DictConfigurator__configure_handler__config_as__HandlerConfiguration_wrong"
# subject = "logging.config.DictConfigurator.configure_handler(config: _HandlerConfiguration)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/logging/config.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: logging.config.DictConfigurator.configure_handler(config: _HandlerConfiguration); call it with the wrong type.

typeshed contract: config is _HandlerConfiguration. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from logging.config import DictConfigurator
obj = object.__new__(DictConfigurator)
try:
    obj.configure_handler(_W())  # config: _HandlerConfiguration <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
