# Helper module for import conformance tests (#1190).
# This file is imported by test_import.py.

MODULE_VAR = 42
GREETING = "hello from helper"

def add(a, b):
    return a + b

def multiply(x, y):
    return x * y

PI = 3

def get_greeting():
    return GREETING
