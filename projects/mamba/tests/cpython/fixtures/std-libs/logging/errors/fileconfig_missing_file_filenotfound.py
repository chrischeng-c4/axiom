# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "errors"
# case = "fileconfig_missing_file_filenotfound"
# subject = "logging.config.fileConfig"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""logging.config.fileConfig: fileConfig on a path that does not exist (inside a TemporaryDirectory) raises FileNotFoundError"""
import logging.config

import os
import tempfile

with tempfile.TemporaryDirectory() as d:
    missing = os.path.join(d, "does_not_exist.ini")
    _raised = False
    try:
        logging.config.fileConfig(missing)
    except FileNotFoundError:
        _raised = True
    assert _raised, "missing config should raise FileNotFoundError"
print("fileconfig_missing_file_filenotfound OK")
