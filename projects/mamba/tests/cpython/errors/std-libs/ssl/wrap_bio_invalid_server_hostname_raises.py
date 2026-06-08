# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "errors"
# case = "wrap_bio_invalid_server_hostname_raises"
# subject = "ssl.SSLContext.wrap_bio"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.SSLContext.wrap_bio: wrap_bio validates server_hostname: an empty string and a leading-dot name each raise ValueError, while an embedded NUL byte raises TypeError"""
import ssl

_ctx = ssl.create_default_context()

# Empty hostname -> ValueError.
try:
    _ctx.wrap_bio(ssl.MemoryBIO(), ssl.MemoryBIO(), server_hostname="")
    raise AssertionError("empty server_hostname should raise")
except ValueError:
    pass

# Leading-dot hostname -> ValueError (UnicodeError is a ValueError subclass).
try:
    _ctx.wrap_bio(ssl.MemoryBIO(), ssl.MemoryBIO(), server_hostname=".example.org")
    raise AssertionError("leading-dot hostname should raise")
except ValueError:
    pass

# Embedded NUL byte -> TypeError.
try:
    _ctx.wrap_bio(ssl.MemoryBIO(), ssl.MemoryBIO(),
                  server_hostname="example.org\x00evil.com")
    raise AssertionError("NUL in hostname should raise")
except TypeError:
    pass

print("wrap_bio_invalid_server_hostname_raises OK")
