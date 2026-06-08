# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "behavior"
# case = "configurator_resolves_cfg_refs"
# subject = "logging.config.BaseConfigurator"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""logging.config.BaseConfigurator: BaseConfigurator.convert resolves cfg:// references by index and key: tuple/list indices, nested index, dotted key, and bracketed key all resolve into the backing dict"""
import logging.config

data = {
    "atuple": (1, 2, 3),
    "alist": ["a", "b", "c"],
    "adict": {"d": "e", "f": 3},
    "nest": ("g", ("h", "i"), "j"),
}
bc = logging.config.BaseConfigurator(data)
assert bc.convert("cfg://atuple[1]") == 2, "tuple index"
assert bc.convert("cfg://alist[1]") == "b", "list index"
assert bc.convert("cfg://nest[1][0]") == "h", "nested index"
assert bc.convert("cfg://adict.d") == "e", "dotted key"
assert bc.convert("cfg://adict[f]") == 3, "bracketed key"
print("configurator_resolves_cfg_refs OK")
