# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "behavior"
# case = "utils_formataddr_roundtrip"
# subject = "email.utils.formataddr"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_email.py"
# status = "filled"
# ///
"""email.utils.formataddr: formataddr((name, addr)) emits a display string carrying both the name and the address"""
import email.utils

formatted = email.utils.formataddr(("Alice Smith", "alice@example.com"))
assert "Alice Smith" in formatted, f"formataddr name = {formatted!r}"
assert "alice@example.com" in formatted, f"formataddr addr = {formatted!r}"

print("utils_formataddr_roundtrip OK")
