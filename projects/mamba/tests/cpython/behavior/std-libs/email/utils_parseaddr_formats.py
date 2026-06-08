# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "behavior"
# case = "utils_parseaddr_formats"
# subject = "email.utils.parseaddr"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_email.py"
# status = "filled"
# ///
"""email.utils.parseaddr: parseaddr splits several address spellings (bare, named-angle, quoted-name) into the expected (realname, addr) pairs"""
import email.utils

cases = [
    ("Alice <alice@example.com>", ("Alice", "alice@example.com")),
    ("bob@example.com", ("", "bob@example.com")),
    ('"Carol Smith" <carol@example.com>', ("Carol Smith", "carol@example.com")),
]
for src, (exp_name, exp_addr) in cases:
    name, addr = email.utils.parseaddr(src)
    assert addr == exp_addr, f"parseaddr addr for {src!r}: {addr!r}"
    assert name == exp_name, f"parseaddr name for {src!r}: {name!r}"

print("utils_parseaddr_formats OK")
