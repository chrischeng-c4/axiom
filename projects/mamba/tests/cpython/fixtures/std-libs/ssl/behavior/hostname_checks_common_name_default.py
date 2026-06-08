# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "behavior"
# case = "hostname_checks_common_name_default"
# subject = "ssl.SSLContext.hostname_checks_common_name"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.SSLContext.hostname_checks_common_name: a client context's hostname_checks_common_name defaults to True"""
import ssl

assert ssl.SSLContext(ssl.PROTOCOL_TLS_CLIENT).hostname_checks_common_name is True, \
    "hostname_checks_common_name default True"

print("hostname_checks_common_name_default OK")
