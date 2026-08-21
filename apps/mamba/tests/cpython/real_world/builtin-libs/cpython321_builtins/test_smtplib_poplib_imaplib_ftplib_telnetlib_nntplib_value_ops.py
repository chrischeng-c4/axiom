# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "cpython321_builtins"
# dimension = "real_world"
# case = "test_smtplib_poplib_imaplib_ftplib_telnetlib_nntplib_value_ops"
# subject = "cpython321.test_smtplib_poplib_imaplib_ftplib_telnetlib_nntplib_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_smtplib_poplib_imaplib_ftplib_telnetlib_nntplib_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_smtplib_poplib_imaplib_ftplib_telnetlib_nntplib_value_ops: execute CPython 3.12 seed test_smtplib_poplib_imaplib_ftplib_telnetlib_nntplib_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Atomic 223 pass conformance — smtplib/poplib/imaplib/ftplib/
# telnetlib/nntplib hasattr contracts that match between
# CPython 3.12 and mamba.
import warnings
warnings.filterwarnings("ignore", category=DeprecationWarning)
import smtplib
import poplib
import imaplib
import ftplib
import telnetlib
import nntplib

_ledger: list[int] = []

# 1) smtplib — full hasattr surface
assert hasattr(smtplib, "SMTP") == True; _ledger.append(1)
assert hasattr(smtplib, "SMTP_SSL") == True; _ledger.append(1)
assert hasattr(smtplib, "SMTP_PORT") == True; _ledger.append(1)
assert hasattr(smtplib, "SMTP_SSL_PORT") == True; _ledger.append(1)
assert hasattr(smtplib, "LMTP") == True; _ledger.append(1)
assert hasattr(smtplib, "LMTP_PORT") == True; _ledger.append(1)
assert hasattr(smtplib, "SMTPException") == True; _ledger.append(1)
assert hasattr(smtplib, "SMTPServerDisconnected") == True; _ledger.append(1)
assert hasattr(smtplib, "SMTPResponseException") == True; _ledger.append(1)
assert hasattr(smtplib, "SMTPSenderRefused") == True; _ledger.append(1)
assert hasattr(smtplib, "SMTPRecipientsRefused") == True; _ledger.append(1)
assert hasattr(smtplib, "SMTPDataError") == True; _ledger.append(1)
assert hasattr(smtplib, "SMTPConnectError") == True; _ledger.append(1)
assert hasattr(smtplib, "SMTPHeloError") == True; _ledger.append(1)
assert hasattr(smtplib, "SMTPAuthenticationError") == True; _ledger.append(1)
assert hasattr(smtplib, "SMTPNotSupportedError") == True; _ledger.append(1)
assert hasattr(smtplib, "quoteaddr") == True; _ledger.append(1)
assert hasattr(smtplib, "quotedata") == True; _ledger.append(1)

# 2) poplib — full hasattr surface
assert hasattr(poplib, "POP3") == True; _ledger.append(1)
assert hasattr(poplib, "POP3_SSL") == True; _ledger.append(1)
assert hasattr(poplib, "POP3_PORT") == True; _ledger.append(1)
assert hasattr(poplib, "POP3_SSL_PORT") == True; _ledger.append(1)
assert hasattr(poplib, "error_proto") == True; _ledger.append(1)
assert hasattr(poplib, "CR") == True; _ledger.append(1)
assert hasattr(poplib, "LF") == True; _ledger.append(1)
assert hasattr(poplib, "CRLF") == True; _ledger.append(1)

# 3) imaplib — conformant hasattr subset
assert hasattr(imaplib, "IMAP4") == True; _ledger.append(1)
assert hasattr(imaplib, "IMAP4_SSL") == True; _ledger.append(1)
assert hasattr(imaplib, "IMAP4_PORT") == True; _ledger.append(1)
assert hasattr(imaplib, "IMAP4_SSL_PORT") == True; _ledger.append(1)
assert hasattr(imaplib, "Internaldate2tuple") == True; _ledger.append(1)
assert hasattr(imaplib, "Int2AP") == True; _ledger.append(1)
assert hasattr(imaplib, "ParseFlags") == True; _ledger.append(1)
assert hasattr(imaplib, "Time2Internaldate") == True; _ledger.append(1)

# 4) ftplib — conformant hasattr subset
assert hasattr(ftplib, "FTP") == True; _ledger.append(1)
assert hasattr(ftplib, "FTP_TLS") == True; _ledger.append(1)
assert hasattr(ftplib, "FTP_PORT") == True; _ledger.append(1)
assert hasattr(ftplib, "MSG_OOB") == True; _ledger.append(1)
assert hasattr(ftplib, "MAXLINE") == True; _ledger.append(1)
assert hasattr(ftplib, "error_reply") == True; _ledger.append(1)
assert hasattr(ftplib, "error_temp") == True; _ledger.append(1)
assert hasattr(ftplib, "error_perm") == True; _ledger.append(1)
assert hasattr(ftplib, "error_proto") == True; _ledger.append(1)
assert hasattr(ftplib, "all_errors") == True; _ledger.append(1)

# 5) telnetlib — conformant hasattr subset
assert hasattr(telnetlib, "Telnet") == True; _ledger.append(1)
assert hasattr(telnetlib, "TELNET_PORT") == True; _ledger.append(1)
assert hasattr(telnetlib, "IAC") == True; _ledger.append(1)
assert hasattr(telnetlib, "DONT") == True; _ledger.append(1)
assert hasattr(telnetlib, "DO") == True; _ledger.append(1)
assert hasattr(telnetlib, "WONT") == True; _ledger.append(1)
assert hasattr(telnetlib, "WILL") == True; _ledger.append(1)
assert hasattr(telnetlib, "SE") == True; _ledger.append(1)
assert hasattr(telnetlib, "NOP") == True; _ledger.append(1)
assert hasattr(telnetlib, "DM") == True; _ledger.append(1)
assert hasattr(telnetlib, "BRK") == True; _ledger.append(1)
assert hasattr(telnetlib, "IP") == True; _ledger.append(1)
assert hasattr(telnetlib, "AO") == True; _ledger.append(1)
assert hasattr(telnetlib, "AYT") == True; _ledger.append(1)
assert hasattr(telnetlib, "EC") == True; _ledger.append(1)
assert hasattr(telnetlib, "EL") == True; _ledger.append(1)
assert hasattr(telnetlib, "GA") == True; _ledger.append(1)
assert hasattr(telnetlib, "SB") == True; _ledger.append(1)

# 6) nntplib — full hasattr surface
assert hasattr(nntplib, "NNTP") == True; _ledger.append(1)
assert hasattr(nntplib, "NNTP_SSL") == True; _ledger.append(1)
assert hasattr(nntplib, "NNTP_PORT") == True; _ledger.append(1)
assert hasattr(nntplib, "NNTP_SSL_PORT") == True; _ledger.append(1)
assert hasattr(nntplib, "NNTPError") == True; _ledger.append(1)
assert hasattr(nntplib, "NNTPReplyError") == True; _ledger.append(1)
assert hasattr(nntplib, "NNTPTemporaryError") == True; _ledger.append(1)
assert hasattr(nntplib, "NNTPPermanentError") == True; _ledger.append(1)
assert hasattr(nntplib, "NNTPProtocolError") == True; _ledger.append(1)
assert hasattr(nntplib, "NNTPDataError") == True; _ledger.append(1)
assert hasattr(nntplib, "decode_header") == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_smtplib_poplib_imaplib_ftplib_telnetlib_nntplib_value_ops {sum(_ledger)} asserts")
