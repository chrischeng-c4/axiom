# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "smtpd"
# dimension = "type"
# case = "SMTPServer__handle_accepted__conn_as_socket_wrong"
# subject = "smtpd.SMTPServer.handle_accepted(conn: socket)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/smtpd.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: smtpd.SMTPServer.handle_accepted(conn: socket); call it with the wrong type.

typeshed contract: conn is socket. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from smtpd import SMTPServer
obj = object.__new__(SMTPServer)
try:
    obj.handle_accepted(_W(), None)  # conn: socket <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
