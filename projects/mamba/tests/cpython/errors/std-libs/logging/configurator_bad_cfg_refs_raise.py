# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "errors"
# case = "configurator_bad_cfg_refs_raise"
# subject = "logging.config.BaseConfigurator"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""logging.config.BaseConfigurator: bad cfg:// references raise documented errors: an unknown top-level key -> KeyError, a malformed prefix 'cfg://!' -> ValueError, and an out-of-range bracket index -> KeyError"""
import logging.config

data = {"adict": {"d": "e"}}
bc = logging.config.BaseConfigurator(data)
for ref, exc in [("cfg://nosuch", KeyError),
                 ("cfg://!", ValueError),
                 ("cfg://adict[2]", KeyError)]:
    _raised = False
    try:
        bc.convert(ref)
    except exc:
        _raised = True
    assert _raised, f"{ref} should raise {exc.__name__}"
print("configurator_bad_cfg_refs_raise OK")
