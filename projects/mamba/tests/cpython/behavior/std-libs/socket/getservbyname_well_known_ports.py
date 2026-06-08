# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "behavior"
# case = "getservbyname_well_known_ports"
# subject = "socket.getservbyname"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""socket.getservbyname: getservbyname maps well-known service names to their IANA ports: http=80, https=443, ftp=21, ssh=22"""
import socket

assert socket.getservbyname("http") == 80, "http = 80"
assert socket.getservbyname("https") == 443, "https = 443"
assert socket.getservbyname("ftp") == 21, "ftp = 21"
assert socket.getservbyname("ssh") == 22, "ssh = 22"
print("getservbyname_well_known_ports OK")
