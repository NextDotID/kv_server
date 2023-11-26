use libsecp256k1::PublicKey;
use sha3::{Digest, Keccak256};

/// Returns compressed public key (in hexstring, without `0x`).
pub fn compress_public_key(pk: &PublicKey) -> String {
    let compressed = pk.serialize_compressed();
    hex::encode(compressed)
}

/// Serialize uncompressed public key (in hexstring, without `0x`).
pub fn hex_public_key(pk: &PublicKey) -> String {
    hex::encode(pk.serialize())
}

/// Keccak256(message)
/// # Example
///
/// ```rust
/// # use kv_server::crypto::util::hash_keccak256;
/// # use hex_literal::hex;
/// #
/// let result = hash_keccak256("Test123");
/// let expected: [u8; 32] = hex!("504AF7475B7341893F803C8EBABFBAEA60EAE7B6A42CB006960C3FDB14DCF8AD");
/// assert_eq!(result, expected);
/// ```
pub fn hash_keccak256(message: &str) -> [u8; 32] {
    let mut hasher = Keccak256::default();
    hasher.update(message);
    hasher.finalize().into()
}

/// SHA256(message)
/// # Example
///
/// ```rust
/// # use kv_server::crypto::util::hash_sha256;
/// # use hex_literal::hex;
/// let result = hash_sha256("Test123");
/// let expected: [u8; 32] = hex!("d9b5f58f0b38198293971865a14074f59eba3e82595becbe86ae51f1d9f1f65e");
/// assert_eq!(result, expected);
/// ```
pub fn hash_sha256(message: &str) -> [u8; 32] {
    let mut context = sha2::Sha256::new();
    context.update(message.as_bytes());
    let mut result: [u8; 32] = [0; 32];
    result.copy_from_slice(context.finalize().as_ref());
    result
}
