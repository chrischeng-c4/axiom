//! JWT (JSON Web Token) encode, decode, and verification.

use crate::CryptoError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Supported JWT signing algorithms.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JwtAlgorithm {
    HS256,
    HS384,
    HS512,
}

impl JwtAlgorithm {
    fn to_jsonwebtoken(&self) -> jsonwebtoken::Algorithm {
        match self {
            Self::HS256 => jsonwebtoken::Algorithm::HS256,
            Self::HS384 => jsonwebtoken::Algorithm::HS384,
            Self::HS512 => jsonwebtoken::Algorithm::HS512,
        }
    }

    /// Parse from string (case-insensitive).
    pub fn from_str(s: &str) -> Result<Self, CryptoError> {
        match s.to_uppercase().as_str() {
            "HS256" => Ok(Self::HS256),
            "HS384" => Ok(Self::HS384),
            "HS512" => Ok(Self::HS512),
            _ => Err(CryptoError::Jwt(format!("unsupported algorithm: {s}"))),
        }
    }
}

/// JWT claims as a flexible JSON map.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Standard claims merged with custom claims.
    #[serde(flatten)]
    pub data: HashMap<String, serde_json::Value>,
}

/// Encode a JWT token from claims.
pub fn jwt_encode(
    claims: &HashMap<String, serde_json::Value>,
    secret: &str,
    algorithm: JwtAlgorithm,
) -> Result<String, CryptoError> {
    let header = jsonwebtoken::Header::new(algorithm.to_jsonwebtoken());
    let claims = Claims {
        data: claims.clone(),
    };
    jsonwebtoken::encode(
        &header,
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| CryptoError::Jwt(e.to_string()))
}

/// Decode and verify a JWT token.
pub fn jwt_decode(
    token: &str,
    secret: &str,
    algorithm: JwtAlgorithm,
) -> Result<HashMap<String, serde_json::Value>, CryptoError> {
    let mut validation = jsonwebtoken::Validation::new(algorithm.to_jsonwebtoken());
    validation.required_spec_claims.clear(); // Don't require exp by default
    let data = jsonwebtoken::decode::<Claims>(
        token,
        &jsonwebtoken::DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .map_err(|e| CryptoError::Jwt(e.to_string()))?;
    Ok(data.claims.data)
}

/// Decode a JWT token WITHOUT verifying the signature.
/// Useful for inspecting token contents.
pub fn jwt_decode_insecure(
    token: &str,
) -> Result<HashMap<String, serde_json::Value>, CryptoError> {
    let mut validation = jsonwebtoken::Validation::default();
    validation.insecure_disable_signature_validation();
    validation.required_spec_claims.clear();
    // Use a dummy key since we're not verifying
    let data = jsonwebtoken::decode::<Claims>(
        token,
        &jsonwebtoken::DecodingKey::from_secret(b""),
        &validation,
    )
    .map_err(|e| CryptoError::Jwt(e.to_string()))?;
    Ok(data.claims.data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jwt_roundtrip() {
        let mut claims = HashMap::new();
        claims.insert("sub".to_string(), serde_json::json!("user123"));
        claims.insert("role".to_string(), serde_json::json!("admin"));

        let token = jwt_encode(&claims, "secret", JwtAlgorithm::HS256).unwrap();
        let decoded = jwt_decode(&token, "secret", JwtAlgorithm::HS256).unwrap();

        assert_eq!(decoded["sub"], "user123");
        assert_eq!(decoded["role"], "admin");
    }

    #[test]
    fn test_jwt_wrong_secret() {
        let mut claims = HashMap::new();
        claims.insert("sub".to_string(), serde_json::json!("user"));
        let token = jwt_encode(&claims, "secret", JwtAlgorithm::HS256).unwrap();
        assert!(jwt_decode(&token, "wrong", JwtAlgorithm::HS256).is_err());
    }

    #[test]
    fn test_jwt_decode_insecure() {
        let mut claims = HashMap::new();
        claims.insert("sub".to_string(), serde_json::json!("user"));
        let token = jwt_encode(&claims, "secret", JwtAlgorithm::HS256).unwrap();
        let decoded = jwt_decode_insecure(&token).unwrap();
        assert_eq!(decoded["sub"], "user");
    }
}
