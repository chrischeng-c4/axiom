# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_robotparser"
# dimension = "behavior"
# case = "crawl_delay_and_custom_agent_test__test_site_maps"
# subject = "cpython.test_robotparser.CrawlDelayAndCustomAgentTest.test_site_maps"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_robotparser.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_robotparser.py::CrawlDelayAndCustomAgentTest::test_site_maps
"""Auto-ported test: CrawlDelayAndCustomAgentTest::test_site_maps (CPython 3.12 oracle)."""


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
robots_txt = '# robots.txt for http://www.example.com/\n\nUser-agent: *\nCrawl-delay: 1\nRequest-rate: 3/15\nDisallow: /cyberworld/map/ # This is an infinite virtual URL space\n\n# Cybermapper knows where to go.\nUser-agent: cybermapper\nDisallow:\n    '
good = ['/', '/test.html', ('cybermapper', '/cyberworld/map/index.html')]
bad = ['/cyberworld/map/index.html']

def get_agent_and_url(url):
    if isinstance(url, tuple):
        agent, url = url
        return (agent, url)
    return (agent, url)
lines = io.StringIO(robots_txt).readlines()
self_parser = urllib.robotparser.RobotFileParser()
self_parser.parse(lines)

assert self_parser.site_maps() == site_maps
print("CrawlDelayAndCustomAgentTest::test_site_maps: ok")
