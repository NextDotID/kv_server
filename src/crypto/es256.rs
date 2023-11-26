use ecdsa::RecoveryId;
use p256::{PublicKey, ecdsa::Signature};


use crate::{error::Error, crypto::util::hash_sha256};

pub fn public_key_from_hex(pubkey_hex: &str) -> Result<PublicKey, Error> {
    let pubkey_bytes = hex::decode(pubkey_hex)?;
    if pubkey_bytes.len() != 64 {
        return Err(Error::ParamError(format!("public key of es256 mismatch: expect 64B, got {}B", pubkey_bytes.len())));
    };

    PublicKey::from_sec1_bytes(pubkey_bytes.as_slice()).map_err(|err| err.into())
}

pub fn validate_sig(pubkey_hex: &str, sign_payload: &str, signature: Vec<u8>) -> Result<PublicKey, Error> {
    let pubkey = public_key_from_hex(pubkey_hex)?;
    let signature = Signature::from_slice(signature.as_ref())?;
    let payload_hash = hash_sha256(sign_payload);

    let _recovery_id = RecoveryId::trial_recovery_from_prehash(&pubkey.into(), payload_hash.as_slice(), &signature)?;
    Ok(pubkey)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_name() {

    }
}
