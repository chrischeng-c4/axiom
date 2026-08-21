# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_robotparser"
# dimension = "behavior"
# case = "string_formatting_test__test_bad_urls"
# subject = "cpython.test_robotparser.StringFormattingTest.test_bad_urls"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_robotparser.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_robotparser.py::StringFormattingTest::test_bad_urls
"""Auto-ported test: StringFormattingTest::test_bad_urls (CPython 3.12 oracle)."""


import io
import os
import threading
import unittest
import urllib.robotparser
from test import support
from test.support import socket_helper
from test.support import threading_helper
from http.server import BaseHTTPRequestHandler, HTTPServer


class RobotHandler(BaseHTTPRequestHandler):

    def do_GET(self):
        self.send_error(403, 'Forbidden access')

    def log_message(self, format, *args):
        pass


# --- test body ---
robots_txt = ''
agent = 'test_robotparser'
good = []
bad = []
site_maps = None
robots_txt = 'User-agent: *\nCrawl-delay: 1\nRequest-rate: 3/15\nDisallow: /cyberworld/map/ # This is an infinite virtual URL space\n\n# Cybermapper knows where to go.\nUser-agent: cybermapper\nDisallow: /some/path\n    '
expected_output = 'User-agent: cybermapper\nDisallow: /some/path\n\nUser-agent: *\nCrawl-delay: 1\nRequest-rate: 3/15\nDisallow: /cyberworld/map/'

def get_agent_and_url(url):
    if isinstance(url, tuple):
        agent, url = url
        return (agent, url)
    return (agent, url)
lines = io.StringIO(robots_txt).readlines()
self_parser = urllib.robotparser.RobotFileParser()
self_parser.parse(lines)
for url in bad:
    agent, url = get_agent_and_url(url)

    assert not self_parser.can_fetch(agent, url)
print("StringFormattingTest::test_bad_urls: ok")
