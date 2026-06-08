# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "behavior"
# case = "buffering_formatter_test__test_default"
# subject = "cpython.test_logging.BufferingFormatterTest.test_default"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_logging.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import logging
import logging.handlers
import logging.config
import codecs
import configparser
import copy
import datetime
import pathlib
import pickle
import io
import itertools
import gc
import json
import os
import queue
import random
import re
import shutil
import socket
import struct
import sys
import tempfile
import textwrap
import threading
import asyncio
import time
import warnings
import weakref
from http.server import HTTPServer, BaseHTTPRequestHandler
from urllib.parse import urlparse, parse_qs
from socketserver import ThreadingUDPServer, DatagramRequestHandler, ThreadingTCPServer, StreamRequestHandler
self_records = [logging.makeLogRecord({'msg': 'one'}), logging.makeLogRecord({'msg': 'two'})]
f = logging.BufferingFormatter()
assert '' == f.format([])
assert 'onetwo' == f.format(self_records)

print("BufferingFormatterTest::test_default: ok")
