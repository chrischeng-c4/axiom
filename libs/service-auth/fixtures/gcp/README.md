# Throwaway test material — not a secret

`throwaway-signing-key.pem` is a real RSA private key and it is **deliberately
public**. It exists so the `gcp` module's unit tests can mint a correctly shaped
Google ID token and verify it offline against `throwaway-jwks.json`, which is the
matching public half in JWKS form. Both are `include_str!`'d at compile time by
`src/gcp.rs`.

It authorizes nothing. It was generated for this test file, has never signed
anything outside it, and is not registered as a signing key for any Google
project, service account, or issuer. Verification in those tests succeeds only
because the test also supplies the matching JWKS — production code fetches
Google's real JWKS, which this key does not appear in.

This is the only private key tracked in the repository. If you are adding a
second one, that is the moment to ask whether it also has to be, rather than
treating this file as precedent.

To regenerate the pair:

```bash
openssl genpkey -algorithm RSA -pkeyopt rsa_keygen_bits:2048 \
  -out throwaway-signing-key.pem
# then re-derive throwaway-jwks.json's n/e from the public half; the `kid` is
# arbitrary but must match the token header the tests build.
```
