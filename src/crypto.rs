//! Cryptographic operations — RSA signing and base64 encoding.
//!
//! Uses the `rsa` crate (RustCrypto) for PKCS1v15 + SHA-256 signing.
//! Replaces the need for Python's `cryptography` library or `openssl` CLI.

use base64::{engine::general_purpose::STANDARD, Engine};
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use rsa::pkcs8::DecodePrivateKey;
use rsa::sha2::Sha256;
use rsa::signature::{SignatureEncoding, Signer};
use rsa::{pkcs1v15::SigningKey, RsaPrivateKey};

/// Sign data with an RSA private key using PKCS1v15 + SHA-256.
///
/// Equivalent to: `openssl dgst -sha256 -sign key.pem data.txt | base64`
///
/// Args:
///     key_path: Path to a PEM-encoded PKCS#8 private key file.
///     data: The string data to sign.
///
/// Returns:
///     Base64-encoded signature.
#[pyfunction]
#[pyo3(signature = (key_path, data))]
pub fn sign_rsa_sha256(key_path: &str, data: &str) -> PyResult<String> {
    dim_log!("[Crypto] Signing {} bytes with {}", data.len(), key_path);

    let pem = std::fs::read_to_string(key_path)
        .map_err(|e| PyRuntimeError::new_err(format!("Failed to read key file: {e}")))?;

    let private_key = RsaPrivateKey::from_pkcs8_pem(&pem)
        .map_err(|e| PyRuntimeError::new_err(format!("Failed to parse private key: {e}")))?;

    let signing_key = SigningKey::<Sha256>::new(private_key);

    let signature = signing_key
        .try_sign(data.as_bytes())
        .map_err(|e| PyRuntimeError::new_err(format!("Signing failed: {e}")))?;

    Ok(STANDARD.encode(signature.to_bytes()))
}

/// Base64-encode a string.
#[pyfunction]
pub fn base64_encode(data: &str) -> String {
    STANDARD.encode(data.as_bytes())
}

/// Base64-decode a string.
#[pyfunction]
pub fn base64_decode(data: &str) -> PyResult<String> {
    let bytes = STANDARD
        .decode(data)
        .map_err(|e| PyRuntimeError::new_err(format!("Base64 decode failed: {e}")))?;
    String::from_utf8(bytes)
        .map_err(|e| PyRuntimeError::new_err(format!("UTF-8 decode failed: {e}")))
}
