# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "errors"
# case = "fileconfig_empty_file_runtimeerror"
# subject = "logging.config.fileConfig"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""logging.config.fileConfig: fileConfig on an empty .ini config file (created inside a TemporaryDirectory) raises RuntimeError"""
import logging.config

import os
import tempfile

with tempfile.TemporaryDirectory() as d:
    path = os.path.join(d, "empty.ini")
    with open(path, "w", encoding="utf-8") as f:
        pass
    _raised = False
    try:
        logging.config.fileConfig(path)
    except RuntimeError:
        _raised = True
    assert _raised, "empty config should raise RuntimeError"
print("fileconfig_empty_file_runtimeerror OK")
