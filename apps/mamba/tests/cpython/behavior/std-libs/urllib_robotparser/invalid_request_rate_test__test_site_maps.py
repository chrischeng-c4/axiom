# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_robotparser"
# dimension = "behavior"
# case = "invalid_request_rate_test__test_site_maps"
# subject = "cpython.test_robotparser.InvalidRequestRateTest.test_site_maps"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_robotparser.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_robotparser.py::InvalidRequestRateTest::test_site_maps
"""Auto-ported test: InvalidRequestRateTest::test_site_maps (CPython 3.12 oracle)."""


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
robots_txt = 'User-agent: *\nDisallow: /tmp/\nDisallow: /a%3Cd.html\nDisallow: /a/b.html\nDisallow: /%7ejoe/index.html\nCrawl-delay: 3\nRequest-rate: 9/banana\n    '
good = ['/tmp']
bad = ['/tmp/', '/tmp/a.html', '/a%3cd.html', '/a%3Cd.html', '/a/b.html', '/%7Ejoe/index.html']
crawl_delay = 3

def get_agent_and_url(url):
    if isinstance(url, tuple):
        agent, url = url
        return (agent, url)
    return (agent, url)
lines = io.StringIO(robots_txt).readlines()
self_parser = urllib.robotparser.RobotFileParser()
self_parser.parse(lines)

assert self_parser.site_maps() == site_maps
print("InvalidRequestRateTest::test_site_maps: ok")
