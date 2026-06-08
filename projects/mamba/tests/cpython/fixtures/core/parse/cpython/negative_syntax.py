# RUN: parse
# EXPECT-ERROR: expected :
# Negative parse tests: syntax errors that MUST be rejected (#566)
# This file intentionally contains invalid Python syntax.
# The parser should reject it with an error containing "expected :".

# Missing colon after if
if True
    pass
