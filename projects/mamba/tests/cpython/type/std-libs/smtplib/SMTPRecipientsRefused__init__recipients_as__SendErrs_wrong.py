# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "smtplib"
# dimension = "type"
# case = "SMTPRecipientsRefused__init__recipients_as__SendErrs_wrong"
# subject = "smtplib.SMTPRecipientsRefused.__init__(recipients: _SendErrs)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/smtplib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: smtplib.SMTPRecipientsRefused.__init__(recipients: _SendErrs); call it with the wrong type.

typeshed contract: recipients is _SendErrs. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from smtplib import SMTPRecipientsRefused
try:
    SMTPRecipientsRefused(_W())  # recipients: _SendErrs <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
