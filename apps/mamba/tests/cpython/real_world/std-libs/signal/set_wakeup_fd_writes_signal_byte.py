# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "real_world"
# case = "set_wakeup_fd_writes_signal_byte"
# subject = "signal.set_wakeup_fd"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""signal.set_wakeup_fd: event-loop wakeup pattern: register a socketpair write end via set_wakeup_fd, raise SIGINT under a no-op handler, and observe the one wakeup byte (the signal number) on the read end; restore the previous fd and handler afterwards"""
import signal
import socket
import struct

SIG = signal.SIGINT

# A no-op handler keeps the signal from terminating us; the wakeup byte is
# written regardless of what the handler does.
old_handler = signal.signal(SIG, lambda s, f: None)

read_sock, write_sock = socket.socketpair()
write_sock.setblocking(False)

# set_wakeup_fd returns the previously registered fd (-1 when none).
prev_fd = signal.set_wakeup_fd(write_sock.fileno())
assert isinstance(prev_fd, int), f"prev fd type = {type(prev_fd)!r}"

try:
    signal.raise_signal(SIG)
    data = read_sock.recv(1)
    assert len(data) == 1, f"one wakeup byte expected: {data!r}"
    (raised,) = struct.unpack("B", data)
    assert raised == SIG, f"wakeup byte {raised} != {int(SIG)}"
finally:
    signal.set_wakeup_fd(prev_fd)
    signal.signal(SIG, old_handler)
    read_sock.close()
    write_sock.close()

print("set_wakeup_fd_writes_signal_byte OK")
