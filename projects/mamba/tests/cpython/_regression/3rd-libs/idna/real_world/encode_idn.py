"""Encode an internationalized domain name to ASCII via `idna.encode`.

End-user scenario: a downstream tool needs to round-trip an IDN through
its Punycode representation before handing it to a stack that only
accepts ASCII hostnames.

DoD: this script must exit 0 under both CPython and mamba.
"""

import idna

# The input is an IDN written with an escape so the fixture source remains
# pure ASCII while still exercising Unicode-to-Punycode encoding.
domain = "\u4f8b.tw"

encoded = idna.encode(domain)

# idna.encode returns bytes; this domain encodes to an ASCII xn-- label.
assert isinstance(encoded, bytes), f"expected bytes, got {type(encoded).__name__}"
assert encoded == b"xn--fsq.tw", f"unexpected encoding: {encoded!r}"

print("ok:", encoded.decode("ascii"))
