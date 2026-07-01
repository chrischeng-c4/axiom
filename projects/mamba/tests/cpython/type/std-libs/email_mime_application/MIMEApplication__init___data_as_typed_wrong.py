# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_mime_application"
# dimension = "type"
# case = "MIMEApplication__init___data_as_typed_wrong"
# subject = "email.mime.application.MIMEApplication.__init__(_data: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/mime/application.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.mime.application.MIMEApplication.__init__(_data: typed); call it with the wrong type.

typeshed contract: _data is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from email.mime.application import MIMEApplication
try:
    MIMEApplication(_W())  # _data: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
