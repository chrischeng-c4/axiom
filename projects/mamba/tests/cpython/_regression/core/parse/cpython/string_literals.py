# RUN: parse
# CPython-derived: string literal variants

# --- regular strings ---
s = "hello world"
s = 'single quotes'

# --- triple-quoted strings ---
s = """multi
line
string"""

# --- raw strings ---
s = r"no \n escapes"

# --- byte strings ---
s = b"byte string"

# --- f-strings ---
name = "world"
s = f"hello {name}"
s = f"result: {1 + 2}"
s = f"empty {x}"

# --- escape sequences in strings ---
s = "tab\there"
s = "newline\nhere"
s = "quote\"inside"

# --- empty strings ---
s = ""
s = ''
