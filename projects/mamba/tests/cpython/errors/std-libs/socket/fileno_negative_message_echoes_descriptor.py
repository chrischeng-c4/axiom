# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "errors"
# case = "fileno_negative_message_echoes_descriptor"
# subject = "socket.socket"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""socket.socket: socket(fileno=-1) / fileno=-42 raises ValueError whose message names 'negative file descriptor'"""
import socket

for bad in (-1, -42):
    raised = False
    try:
        socket.socket(socket.AF_INET, socket.SOCK_STREAM, fileno=bad)
    except ValueError as e:
        raised = True
        assert "negative file descriptor" in str(e), str(e)
    assert raised, f"fileno={bad} should raise ValueError"
print("fileno_negative_message_echoes_descriptor OK")
