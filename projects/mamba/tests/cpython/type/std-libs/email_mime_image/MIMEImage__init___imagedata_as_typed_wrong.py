# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_mime_image"
# dimension = "type"
# case = "MIMEImage__init___imagedata_as_typed_wrong"
# subject = "email.mime.image.MIMEImage.__init__(_imagedata: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/mime/image.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.mime.image.MIMEImage.__init__(_imagedata: typed); call it with the wrong type.

typeshed contract: _imagedata is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from email.mime.image import MIMEImage
try:
    MIMEImage(_W())  # _imagedata: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
