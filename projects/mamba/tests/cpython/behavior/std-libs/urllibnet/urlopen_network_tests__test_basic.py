# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllibnet"
# dimension = "behavior"
# case = "urlopen_network_tests__test_basic"
# subject = "cpython.test_urllibnet.urlopenNetworkTests.test_basic"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_urllibnet.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_urllibnet.py::urlopenNetworkTests::test_basic
"""Auto-ported test: urlopenNetworkTests::test_basic."""


import os
import urllib.request


if os.environ.get("MAMBA_RUN_NETWORK") != "1":
    print("urlopenNetworkTests::test_basic: skipped; set MAMBA_RUN_NETWORK=1")
    raise SystemExit(0)

url = os.environ.get("MAMBA_TEST_HTTP_URL", "http://www.pythontest.net/")

try:
    with urllib.request.urlopen(url, timeout=15) as open_url:
        for attr in ("read", "readline", "readlines", "fileno", "close", "info", "geturl"):
            assert hasattr(open_url, attr), f"object returned from urlopen lacks {attr}"
        assert open_url.read(), "calling 'read' failed"
finally:
    urllib.request.urlcleanup()

print("urlopenNetworkTests::test_basic: ok")
