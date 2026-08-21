# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "smtplib"
# dimension = "type"
# case = "SMTP__auth_cram_md5__challenge_as_ReadableBuffer_wrong"
# subject = "smtplib.SMTP.auth_cram_md5(challenge: ReadableBuffer)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/smtplib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: smtplib.SMTP.auth_cram_md5(challenge: ReadableBuffer); call it with the wrong type.

typeshed contract: challenge is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from smtplib import SMTP
obj = object.__new__(SMTP)
try:
    obj.auth_cram_md5(_W())  # challenge: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
