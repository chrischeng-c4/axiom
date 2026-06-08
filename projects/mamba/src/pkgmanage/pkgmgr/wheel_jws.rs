// wheel_jws.rs — read JSON-serialized JWS signature files used by wheels.
//
// PEP 491 § "The .dist-info directory" optionally permits signed wheels
// where `.dist-info/RECORD.jws` carries an RFC 7515 JSON Web Signature
// over the canonicalized RECORD contents. uv ignores these files but
// production redistributors (TUF, in-house mirror toolchains) verify
// them, so we parse the on-disk format here and leave signature
// verification to the caller's crypto layer.
//
// RFC 7515 §7.2 defines two JSON shapes:
//
//   * Flattened (single signature, common for wheel RECORD.jws):
//     ```
//     { "protected": "<b64u>", "header": {...},
//       "payload":   "<b64u>", "signature": "<b64u>" }
//     ```
//
//   * General (multi-signature):
//     ```
//     { "payload": "<b64u>",
//       "signatures": [
//         { "protected": "<b64u>", "header": {...}, "signature": "<b64u>" },
//         ...
//       ] }
//     ```
//
// All `<b64u>` fields use the BASE64URL alphabet *without* padding
// (RFC 7515 §2). Empty strings are legal — they represent an empty
// header / payload, which the JOSE spec explicitly permits.
//
// The compact serialization (`protected.payload.signature`) is NOT
// covered here; wheel RECORD.jws is always JSON.

use base64::Engine;

use crate::pkgmanage::pkgmgr::types::IndexError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JwsSignature {
    /// Raw base64url-encoded JWS Protected Header, retained verbatim
    /// because signature verification re-uses it as input. `None` if
    /// the signature has no protected header.
    pub protected_b64: Option<String>,
    /// Decoded JSON of the protected header (`None` if absent).
    pub protected_json: Option<serde_json::Value>,
    /// Unprotected JWS Header object (`None` if absent).
    pub header: Option<serde_json::Value>,
    /// Decoded signature bytes.
    pub signature_bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WheelJws {
    /// Raw base64url-encoded payload, retained for signature
    /// verification (re-used as `b64(protected) + "." + b64(payload)`).
    pub payload_b64: String,
    /// Decoded payload bytes (the canonicalized RECORD).
    pub payload_bytes: Vec<u8>,
    /// One entry per signature. Flattened JWS produces a length-1 vec.
    pub signatures: Vec<JwsSignature>,
}

/// Parse a RECORD.jws JSON document. Accepts both the flattened
/// (single signature) and general (multi-signature) serializations.
pub fn parse_wheel_jws(src: &str) -> Result<WheelJws, IndexError> {
    let root: serde_json::Value = serde_json::from_str(src).map_err(|e| {
        IndexError::ParseError {
            url: String::new(),
            detail: format!("RECORD.jws is not valid JSON: {e}"),
        }
    })?;

    let obj = root.as_object().ok_or_else(|| IndexError::ParseError {
        url: String::new(),
        detail: "RECORD.jws root must be a JSON object".into(),
    })?;

    let payload_b64 = obj
        .get("payload")
        .and_then(|v| v.as_str())
        .ok_or_else(|| IndexError::ParseError {
            url: String::new(),
            detail: "RECORD.jws missing 'payload' string".into(),
        })?
        .to_string();
    let payload_bytes = decode_b64url(&payload_b64, "payload")?;

    let signatures = if let Some(arr) = obj.get("signatures") {
        // General serialization.
        let entries = arr.as_array().ok_or_else(|| IndexError::ParseError {
            url: String::new(),
            detail: "RECORD.jws 'signatures' must be an array".into(),
        })?;
        if entries.is_empty() {
            return Err(IndexError::ParseError {
                url: String::new(),
                detail: "RECORD.jws 'signatures' array is empty".into(),
            });
        }
        entries
            .iter()
            .map(parse_signature_entry)
            .collect::<Result<Vec<_>, _>>()?
    } else if obj.contains_key("signature") {
        // Flattened serialization. `protected`, `header`, `signature`
        // sit at the root.
        vec![parse_signature_entry(&root)?]
    } else {
        return Err(IndexError::ParseError {
            url: String::new(),
            detail: "RECORD.jws missing 'signature' (flattened) or 'signatures' (general)"
                .into(),
        });
    };

    Ok(WheelJws {
        payload_b64,
        payload_bytes,
        signatures,
    })
}

fn parse_signature_entry(v: &serde_json::Value) -> Result<JwsSignature, IndexError> {
    let obj = v.as_object().ok_or_else(|| IndexError::ParseError {
        url: String::new(),
        detail: "JWS signature entry must be a JSON object".into(),
    })?;

    let protected_b64 = obj.get("protected").and_then(|v| v.as_str()).map(String::from);
    let protected_json = match &protected_b64 {
        Some(b64) if b64.is_empty() => None,
        Some(b64) => {
            let bytes = decode_b64url(b64, "protected")?;
            let parsed: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| {
                IndexError::ParseError {
                    url: String::new(),
                    detail: format!("JWS 'protected' header is not valid JSON: {e}"),
                }
            })?;
            Some(parsed)
        }
        None => None,
    };

    let header = obj.get("header").cloned();
    if let Some(h) = &header {
        if !h.is_object() {
            return Err(IndexError::ParseError {
                url: String::new(),
                detail: "JWS 'header' must be a JSON object".into(),
            });
        }
    }

    let signature_b64 = obj
        .get("signature")
        .and_then(|v| v.as_str())
        .ok_or_else(|| IndexError::ParseError {
            url: String::new(),
            detail: "JWS signature entry missing 'signature' string".into(),
        })?;
    let signature_bytes = decode_b64url(signature_b64, "signature")?;
    if signature_bytes.is_empty() {
        return Err(IndexError::ParseError {
            url: String::new(),
            detail: "JWS 'signature' must not decode to empty".into(),
        });
    }

    Ok(JwsSignature {
        protected_b64,
        protected_json,
        header,
        signature_bytes,
    })
}

fn decode_b64url(s: &str, field: &str) -> Result<Vec<u8>, IndexError> {
    let engine = base64::engine::general_purpose::URL_SAFE_NO_PAD;
    engine.decode(s.as_bytes()).map_err(|e| IndexError::ParseError {
        url: String::new(),
        detail: format!("JWS '{field}' base64url decode failed: {e}"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn err_detail(err: IndexError) -> String {
        match err {
            IndexError::ParseError { detail, .. } => detail,
            other => panic!("expected ParseError, got {other:?}"),
        }
    }

    fn b64u(s: &[u8]) -> String {
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(s)
    }

    #[test]
    fn parses_flattened_jws() {
        let protected = br#"{"alg":"RS256"}"#;
        let payload = b"record contents\n";
        let signature = &[0xaa, 0xbb, 0xcc, 0xdd];
        let src = format!(
            r#"{{"protected":"{}","payload":"{}","signature":"{}"}}"#,
            b64u(protected),
            b64u(payload),
            b64u(signature)
        );
        let jws = parse_wheel_jws(&src).unwrap();
        assert_eq!(jws.payload_bytes, payload);
        assert_eq!(jws.signatures.len(), 1);
        let sig = &jws.signatures[0];
        assert_eq!(sig.signature_bytes, signature);
        assert_eq!(
            sig.protected_json.as_ref().unwrap()["alg"].as_str(),
            Some("RS256")
        );
    }

    #[test]
    fn parses_general_serialization_multi_signature() {
        let payload = b"record\n";
        let prot1 = br#"{"alg":"RS256"}"#;
        let prot2 = br#"{"alg":"ES256"}"#;
        let sig1 = &[1, 2, 3];
        let sig2 = &[4, 5, 6, 7];
        let src = format!(
            r#"{{
                "payload": "{}",
                "signatures": [
                    {{"protected":"{}","signature":"{}"}},
                    {{"protected":"{}","signature":"{}"}}
                ]
            }}"#,
            b64u(payload),
            b64u(prot1),
            b64u(sig1),
            b64u(prot2),
            b64u(sig2)
        );
        let jws = parse_wheel_jws(&src).unwrap();
        assert_eq!(jws.payload_bytes, payload);
        assert_eq!(jws.signatures.len(), 2);
        assert_eq!(jws.signatures[0].signature_bytes, sig1);
        assert_eq!(jws.signatures[1].signature_bytes, sig2);
    }

    #[test]
    fn flattened_unprotected_header_preserved() {
        let src = format!(
            r#"{{"protected":"{}","header":{{"kid":"key-1"}},"payload":"{}","signature":"{}"}}"#,
            b64u(br#"{"alg":"none"}"#),
            b64u(b""),
            b64u(&[9])
        );
        let jws = parse_wheel_jws(&src).unwrap();
        assert_eq!(
            jws.signatures[0].header.as_ref().unwrap()["kid"].as_str(),
            Some("key-1")
        );
    }

    #[test]
    fn empty_protected_string_yields_none_json() {
        let src = format!(
            r#"{{"protected":"","payload":"{}","signature":"{}"}}"#,
            b64u(b"x"),
            b64u(&[1])
        );
        let jws = parse_wheel_jws(&src).unwrap();
        assert_eq!(jws.signatures[0].protected_b64.as_deref(), Some(""));
        assert!(jws.signatures[0].protected_json.is_none());
    }

    #[test]
    fn empty_payload_allowed() {
        let src = format!(
            r#"{{"protected":"{}","payload":"","signature":"{}"}}"#,
            b64u(br#"{"alg":"none"}"#),
            b64u(&[1])
        );
        let jws = parse_wheel_jws(&src).unwrap();
        assert!(jws.payload_bytes.is_empty());
        assert_eq!(jws.payload_b64, "");
    }

    #[test]
    fn payload_b64_retained_verbatim() {
        let payload_b64 = b64u(b"hello");
        let src = format!(
            r#"{{"protected":"{}","payload":"{}","signature":"{}"}}"#,
            b64u(br#"{"alg":"none"}"#),
            payload_b64,
            b64u(&[1])
        );
        let jws = parse_wheel_jws(&src).unwrap();
        assert_eq!(jws.payload_b64, b64u(b"hello"));
    }

    #[test]
    fn rejects_invalid_json() {
        let err = parse_wheel_jws("not json").unwrap_err();
        assert!(err_detail(err).contains("not valid JSON"));
    }

    #[test]
    fn rejects_non_object_root() {
        let err = parse_wheel_jws("[]").unwrap_err();
        assert!(err_detail(err).contains("must be a JSON object"));
    }

    #[test]
    fn rejects_missing_payload() {
        let src = format!(
            r#"{{"protected":"{}","signature":"{}"}}"#,
            b64u(br#"{"alg":"none"}"#),
            b64u(&[1])
        );
        let err = parse_wheel_jws(&src).unwrap_err();
        assert!(err_detail(err).contains("missing 'payload'"));
    }

    #[test]
    fn rejects_neither_signature_nor_signatures() {
        let src = format!(r#"{{"payload":"{}"}}"#, b64u(b"x"));
        let err = parse_wheel_jws(&src).unwrap_err();
        assert!(err_detail(err).contains("missing 'signature'"));
    }

    #[test]
    fn rejects_empty_signatures_array() {
        let src = format!(r#"{{"payload":"{}","signatures":[]}}"#, b64u(b"x"));
        let err = parse_wheel_jws(&src).unwrap_err();
        assert!(err_detail(err).contains("'signatures' array is empty"));
    }

    #[test]
    fn rejects_non_array_signatures() {
        let src = format!(r#"{{"payload":"{}","signatures":"oops"}}"#, b64u(b"x"));
        let err = parse_wheel_jws(&src).unwrap_err();
        assert!(err_detail(err).contains("must be an array"));
    }

    #[test]
    fn rejects_signature_entry_missing_signature_field() {
        let src = format!(
            r#"{{"payload":"{}","signatures":[{{"protected":"{}"}}]}}"#,
            b64u(b"x"),
            b64u(br#"{"alg":"none"}"#)
        );
        let err = parse_wheel_jws(&src).unwrap_err();
        assert!(err_detail(err).contains("missing 'signature' string"));
    }

    #[test]
    fn rejects_empty_signature_bytes() {
        let src = format!(
            r#"{{"protected":"{}","payload":"{}","signature":""}}"#,
            b64u(br#"{"alg":"none"}"#),
            b64u(b"x")
        );
        let err = parse_wheel_jws(&src).unwrap_err();
        assert!(err_detail(err).contains("must not decode to empty"));
    }

    #[test]
    fn rejects_protected_with_non_json_payload() {
        let src = format!(
            r#"{{"protected":"{}","payload":"{}","signature":"{}"}}"#,
            b64u(b"not json"),
            b64u(b"x"),
            b64u(&[1])
        );
        let err = parse_wheel_jws(&src).unwrap_err();
        assert!(err_detail(err).contains("'protected' header is not valid JSON"));
    }

    #[test]
    fn rejects_b64url_with_padding() {
        // RFC 7515 §2: NO padding. "Zg==" should fail.
        let src = r#"{"protected":"e30","payload":"Zg==","signature":"AQ"}"#;
        let err = parse_wheel_jws(src).unwrap_err();
        assert!(err_detail(err).contains("base64url decode failed"));
    }

    #[test]
    fn rejects_b64url_with_standard_alphabet() {
        // `+/` standard chars are NOT URL_SAFE; should fail.
        let src = format!(
            r#"{{"protected":"{}","payload":"+/","signature":"{}"}}"#,
            b64u(br#"{"alg":"none"}"#),
            b64u(&[1])
        );
        let err = parse_wheel_jws(&src).unwrap_err();
        assert!(err_detail(err).contains("'payload' base64url"));
    }

    #[test]
    fn rejects_non_object_header() {
        let src = format!(
            r#"{{"protected":"{}","header":"oops","payload":"{}","signature":"{}"}}"#,
            b64u(br#"{"alg":"none"}"#),
            b64u(b"x"),
            b64u(&[1])
        );
        let err = parse_wheel_jws(&src).unwrap_err();
        assert!(err_detail(err).contains("'header' must be a JSON object"));
    }

    #[test]
    fn realistic_wheel_record_jws() {
        // Approximation of a real wheel-redistributor signature: RS256
        // protected header, multi-line RECORD payload, opaque signature.
        let protected = br#"{"alg":"RS256","typ":"JWT"}"#;
        let payload = b"\
mypkg/__init__.py,sha256=abcdef,42
mypkg-1.0.dist-info/METADATA,sha256=123,500
mypkg-1.0.dist-info/RECORD,,
";
        let signature: Vec<u8> = (0..64).collect();
        let src = format!(
            r#"{{"protected":"{}","header":{{"kid":"mirror-2026"}},"payload":"{}","signature":"{}"}}"#,
            b64u(protected),
            b64u(payload),
            b64u(&signature)
        );
        let jws = parse_wheel_jws(&src).unwrap();
        assert_eq!(jws.signatures.len(), 1);
        assert_eq!(jws.payload_bytes, payload);
        let sig = &jws.signatures[0];
        assert_eq!(
            sig.protected_json.as_ref().unwrap()["alg"].as_str(),
            Some("RS256")
        );
        assert_eq!(
            sig.header.as_ref().unwrap()["kid"].as_str(),
            Some("mirror-2026")
        );
        assert_eq!(sig.signature_bytes.len(), 64);
    }
}
