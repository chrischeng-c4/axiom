# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_mime_audio"
# dimension = "type"
# case = "MIMEAudio__init___audiodata_as_typed_wrong"
# subject = "email.mime.audio.MIMEAudio.__init__(_audiodata: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/mime/audio.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.mime.audio.MIMEAudio.__init__(_audiodata: typed); call it with the wrong type.

typeshed contract: _audiodata is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from email.mime.audio import MIMEAudio
try:
    MIMEAudio(_W())  # _audiodata: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
