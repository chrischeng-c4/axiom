# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_robotparser"
# dimension = "behavior"
# case = "user_agent_google_mobile_test__test_site_maps"
# subject = "cpython.test_robotparser.UserAgentGoogleMobileTest.test_site_maps"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_robotparser.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_robotparser.py::UserAgentGoogleMobileTest::test_site_maps
"""Auto-ported test: UserAgentGoogleMobileTest::test_site_maps (CPython 3.12 oracle)."""


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
robots_txt = 'User-agent: Googlebot\nDisallow: /\n\nUser-agent: Googlebot-Mobile\nAllow: /\n    '
agent = 'Googlebot'
bad = ['/something.jpg']
agent = 'Googlebot-Mobile'

def get_agent_and_url(url):
    if isinstance(url, tuple):
        agent, url = url
        return (agent, url)
    return (agent, url)
lines = io.StringIO(robots_txt).readlines()
self_parser = urllib.robotparser.RobotFileParser()
self_parser.parse(lines)

assert self_parser.site_maps() == site_maps
print("UserAgentGoogleMobileTest::test_site_maps: ok")
