use libsecp256k1::PublicKey;
use sha3::{Keccak256, Digest};

/// Returns compressed public key (in hexstring, without `0x`).
pub fn compress_public_key(pk: &PublicKey) -> String {
    let compressed = pk.serialize_compressed();
    hex::encode(compressed)
}

/// Keccak256(message)
/// # Example
///
/// ```rust
/// # use kv_server::crypto::util::hash_keccak256;
/// # use hex_literal::hex;
/// #
/// let result = hash_keccak256(&"Test123");
/// let expected: [u8; 32] = hex!("504AF7475B7341893F803C8EBABFBAEA60EAE7B6A42CB006960C3FDB14DCF8AD");
/// assert_eq!(result, expected);
/// ```
pub fn hash_keccak256(message: &str) -> [u8; 32] {
    let mut hasher = Keccak256::default();
    hasher.update(message);
    hasher.finalize().into()
}
