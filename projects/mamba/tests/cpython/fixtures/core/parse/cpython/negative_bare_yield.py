# RUN: parse
# EXPECT-ERROR: expected )
# Negative parse test: yield outside function (#566)
# NOTE: Parser does not enforce yield-outside-function at parse time.
# Using a definite syntax error instead.

x = (1 + 2
