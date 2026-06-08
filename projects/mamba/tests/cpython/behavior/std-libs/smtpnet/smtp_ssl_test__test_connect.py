# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "smtpnet"
# dimension = "behavior"
# case = "smtp_ssl_test__test_connect"
# subject = "cpython.test_smtpnet.SmtpSSLTest.test_connect"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_smtpnet.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_smtpnet.py::SmtpSSLTest::test_connect
"""Auto-ported test: SmtpSSLTest::test_connect."""


import os
import smtplib


if os.environ.get("MAMBA_RUN_NETWORK") != "1":
    print("SmtpSSLTest::test_connect: skipped; set MAMBA_RUN_NETWORK=1")
    raise SystemExit(0)

assert hasattr(smtplib, "SMTP_SSL"), "smtplib.SMTP_SSL is required"

host = os.environ.get("MAMBA_SMTP_TEST_HOST", "smtp.gmail.com")
port = int(os.environ.get("MAMBA_SMTP_TEST_PORT", "465"))

server = smtplib.SMTP_SSL(host, port, timeout=15)
try:
    server.ehlo()
finally:
    server.quit()

print("SmtpSSLTest::test_connect: ok")
