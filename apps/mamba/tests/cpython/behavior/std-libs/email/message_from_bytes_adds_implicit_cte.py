# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "behavior"
# case = "message_from_bytes_adds_implicit_cte"
# subject = "email.message_from_bytes"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_email.py"
# status = "filled"
# ///
"""email.message_from_bytes: a non-ascii iso-8859-1 body with no declared Content-Transfer-Encoding gains a quoted-printable CTE on as_string(), encoding the high bytes (F=F6=F6 b=E4r)"""
import email

import textwrap

# A non-ascii iso-8859-1 body with no declared Content-Transfer-Encoding gains
# a quoted-printable CTE when re-serialized to a string.
src = textwrap.dedent('''\
MIME-Version: 1.0
Content-type: text/plain; charset="iso-8859-1"

Non-ascii body: F\xf6\xf6 b\xe4r
''').encode("iso-8859-1")
m = email.message_from_bytes(src)
out = m.as_string()
assert "Content-Transfer-Encoding: quoted-printable" in out, "implicit CTE added"
assert "F=F6=F6 b=E4r" in out, f"quoted-printable body: {out[-40:]!r}"

print("message_from_bytes_adds_implicit_cte OK")
