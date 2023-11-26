use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

use crate::{error::Error, util, crypto};

/// Algorithm supported by subkey
#[derive(Display, Debug, Copy, Clone, Serialize, Deserialize, EnumString, PartialEq, Eq)]
pub enum Algorithm {
    /// Secp256k1 curve, signing after Keccak256 hash (personal_sign) using ECDSA
    #[strum(serialize = "secp256k1")]
    #[serde(rename = "secp256k1")]
    Secp256k1,
    /// P-256 (aka Secp256r1) curve, signing after SHA256 hash using ECDSA
    #[strum(serialize = "es256")]
    #[serde(rename = "es256")]
    ES256,
}

impl Algorithm {
    pub fn verify(
        &self,
        public_key: &str,
        sign_payload: &str,
        signature: &str,
    ) -> Result<(), Error> {
        let signature_bytes = parse_signature(signature)?;
        match self {
            Algorithm::Secp256k1 => {
                let public_key = crypto::secp256k1::Secp256k1KeyPair::from_pubkey_hex(public_key)?.public_key;
                let recovered = crypto::secp256k1::Secp256k1KeyPair::recover_from_personal_signature(&signature_bytes, sign_payload)?;
                if public_key == recovered {
                    Ok(())
                } else {
                    Err(Error::SignatureValidationError("Signature not match".into()))
                }
            },
            Algorithm::ES256 => {
                crypto::es256::validate_sig(public_key, sign_payload, signature_bytes).map(|_| ())
            },
        }
    }
}

fn parse_signature(signature: &str) -> Result<Vec<u8>, Error> {
    let result = if signature.starts_with("0x") {
        hex::decode(signature.strip_prefix("0x").unwrap())?
    }else {
        // Base64
        util::base64_to_vec(signature)?
    };

    Ok(result)
}
