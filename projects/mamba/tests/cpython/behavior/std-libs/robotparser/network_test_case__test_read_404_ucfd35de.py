# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "robotparser"
# dimension = "behavior"
# case = "network_test_case__test_read_404_ucfd35de"
# subject = "cpython.test_robotparser.NetworkTestCase.test_read_404"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_robotparser.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import io
import os
import threading
import urllib.robotparser
from http.server import BaseHTTPRequestHandler, HTTPServer
base_url = 'http://www.pythontest.net/'
robots_txt = '{}elsewhere/robots.txt'.format(base_url)

def url(path):
    return '{}{}{}'.format(base_url, path, '/' if not os.path.splitext(path)[1] else '')
parser = urllib.robotparser.RobotFileParser(url('i-robot.txt'))
parser.read()
assert parser.allow_all
assert not parser.disallow_all
assert parser.mtime() == 0
assert parser.crawl_delay('*') is None
assert parser.request_rate('*') is None

print("NetworkTestCase::test_read_404: ok")
