"""
cclab.crypto — Cryptography and authentication primitives.

Hashing:
    sha256, sha512, sha1, md5, blake3 — hash functions (bytes and hex)
    hmac_sha256, hmac_sha512 — HMAC signing and verification

Encryption:
    aes_gcm_encrypt/decrypt — AES-256-GCM authenticated encryption
    chacha20_encrypt/decrypt — ChaCha20-Poly1305 authenticated encryption
    generate_key, generate_nonce — key/nonce generation

Password Hashing:
    bcrypt_hash/verify — bcrypt password hashing
    argon2_hash/verify — Argon2id password hashing

JWT:
    jwt_encode, jwt_decode — JWT token operations
    jwt_decode_insecure — decode without verification

OTP:
    totp_generate/verify — TOTP (Google Authenticator compatible)
    hotp_generate/verify — HOTP (RFC 4226)

Encoding:
    base64_encode/decode, base64url_encode/decode
    hex_encode/decode
    random_bytes, random_hex, random_string
"""

try:
    from cclab.cclab import crypto as _crypto  # type: ignore

    # Hashing
    sha256 = _crypto.sha256
    sha256_hex = _crypto.sha256_hex
    sha512 = _crypto.sha512
    sha512_hex = _crypto.sha512_hex
    sha1 = _crypto.sha1
    sha1_hex = _crypto.sha1_hex
    md5 = _crypto.md5
    md5_hex = _crypto.md5_hex
    blake3 = _crypto.blake3
    blake3_hex = _crypto.blake3_hex
    hmac_sha256 = _crypto.hmac_sha256
    hmac_sha256_hex = _crypto.hmac_sha256_hex
    hmac_sha256_verify = _crypto.hmac_sha256_verify
    hmac_sha512 = _crypto.hmac_sha512
    hmac_sha512_hex = _crypto.hmac_sha512_hex

    # AEAD
    aes_gcm_encrypt = _crypto.aes_gcm_encrypt
    aes_gcm_decrypt = _crypto.aes_gcm_decrypt
    chacha20_encrypt = _crypto.chacha20_encrypt
    chacha20_decrypt = _crypto.chacha20_decrypt
    generate_key = _crypto.generate_key
    generate_nonce = _crypto.generate_nonce

    # Password
    bcrypt_hash = _crypto.bcrypt_hash
    bcrypt_verify = _crypto.bcrypt_verify
    argon2_hash = _crypto.argon2_hash
    argon2_verify = _crypto.argon2_verify

    # JWT
    jwt_encode = _crypto.jwt_encode
    jwt_decode = _crypto.jwt_decode
    jwt_decode_insecure = _crypto.jwt_decode_insecure

    # OTP
    totp_generate = _crypto.totp_generate
    totp_verify = _crypto.totp_verify
    hotp_generate = _crypto.hotp_generate
    hotp_verify = _crypto.hotp_verify

    # Encoding
    base64_encode = _crypto.base64_encode
    base64_decode = _crypto.base64_decode
    base64url_encode = _crypto.base64url_encode
    base64url_decode = _crypto.base64url_decode
    hex_encode = _crypto.hex_encode
    hex_decode = _crypto.hex_decode
    random_bytes = _crypto.random_bytes
    random_hex = _crypto.random_hex
    random_string = _crypto.random_string

except ImportError:
    pass
