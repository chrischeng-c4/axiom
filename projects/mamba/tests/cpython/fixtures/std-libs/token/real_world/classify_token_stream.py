# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "token"
# dimension = "real_world"
# case = "classify_token_stream"
# subject = "token.tok_name"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""token.tok_name: a tokenizer consumer names a stream of token-type ids via tok_name and partitions them by ISTERMINAL/ISEOF"""
import token

# A downstream consumer receives a stream of raw token-type ids (e.g. from a
# parser or tokenizer) and must label and classify them using only the public
# token module surface.
stream = [token.NAME, token.OP, token.NUMBER, token.NT_OFFSET, token.ENDMARKER]

names = [token.tok_name[t] for t in stream]
assert names == ["NAME", "OP", "NUMBER", "NT_OFFSET", "ENDMARKER"], names

terminals = [t for t in stream if token.ISTERMINAL(t)]
nonterminals = [t for t in stream if token.ISNONTERMINAL(t)]
assert [token.tok_name[t] for t in terminals] == ["NAME", "OP", "NUMBER", "ENDMARKER"]
assert [token.tok_name[t] for t in nonterminals] == ["NT_OFFSET"]

# The stream terminates exactly when an ENDMARKER token is reached.
eof_index = next(i for i, t in enumerate(stream) if token.ISEOF(t))
assert eof_index == len(stream) - 1, eof_index

print("classify_token_stream OK")
