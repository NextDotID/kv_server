use crate::{
    crypto::util::{compress_public_key, hash_keccak256},
    error::Error,
};
use libsecp256k1::{Message, PublicKey, RecoveryId, SecretKey, Signature};
use rand::rngs;

/// Supports non-SecretKey usage.
pub struct Secp256k1KeyPair {
    pub public_key: PublicKey,
    pub secret_key: Option<SecretKey>,
}

impl std::fmt::Debug for Secp256k1KeyPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.secret_key {
            Some(_) => f
                .debug_struct("Secp256k1KeyPair")
                .field("public_key", &compress_public_key(&self.public_key))
                .field("secret_key", &"OMIT".to_string())
                .finish(),
            None => f
                .debug_struct("Secp256k1KeyPair")
                .field("public_key", &self.public_key)
                .finish(),
        }
    }
}

impl Secp256k1KeyPair {
    /// Generate a keypair.
    /// For test purpose only.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use kv_server::crypto::secp256k1::Secp256k1KeyPair;
    ///
    /// let keypair = Secp256k1KeyPair::generate();
    /// # assert_eq!(65, keypair.public_key.serialize().len());
    /// # assert_eq!(32, keypair.secret_key.unwrap().serialize().len());
    /// ```
    pub fn generate() -> Self {
        let mut rng = rngs::OsRng;
        let secret_key = SecretKey::random(&mut rng);
        let public_key = PublicKey::from_secret_key(&secret_key);

        Self {
            public_key,
            secret_key: Some(secret_key),
        }
    }

    /// Parse full or compressed pubkey from hexstring. Both `0x...`
    /// and raw hexstring are supported.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use kv_server::crypto::secp256k1::Secp256k1KeyPair;
    /// # use hex_literal::hex;
    /// # let pubkey_hex = "0x04c7cacde73af939c35d527b34e0556ea84bab27e6c0ed7c6c59be70f6d2db59c206b23529977117dc8a5d61fa848f94950422b79d1c142bcf623862e49f9e6575";
    /// let pair = Secp256k1KeyPair::from_pubkey_hex(&pubkey_hex.to_string()).unwrap();
    /// # assert_eq!(hex!("04c7cacde73af939c35d527b34e0556ea84bab27e6c0ed7c6c59be70f6d2db59c206b23529977117dc8a5d61fa848f94950422b79d1c142bcf623862e49f9e6575"), pair.public_key.serialize());
    /// ```
    pub fn from_pubkey_hex(pubkey_hex: &str) -> Result<Self, Error> {
        let hex: &str;
        if pubkey_hex.starts_with("0x") {
            hex = &pubkey_hex[2..];
        } else {
            hex = pubkey_hex;
        };
        let pubkey_bytes = hex::decode(hex).map_err(|e| Error::from(e))?;
        Self::from_pubkey_vec(&pubkey_bytes)
    }

    pub fn from_pubkey_vec(pubkey_vec: &Vec<u8>) -> Result<Self, Error> {
        // `None` will try 65- and 33-bytes parser        vvvv
        let pubkey =
            PublicKey::parse_slice(pubkey_vec.as_slice(), None).map_err(|e| Error::from(e))?;

        Ok(Self {
            public_key: pubkey,
            secret_key: None,
        })
    }

    /// `web3.eth.personal.sign`
    /// # Examples
    ///
    /// ```rust
    /// # use kv_server::crypto::secp256k1::Secp256k1KeyPair;
    /// # use hex_literal::hex;
    /// # use libsecp256k1::{SecretKey, PublicKey};
    /// #
    /// let sign_payload = "Test123!".to_string();
    /// # let secret_key = SecretKey::parse(&hex!("b5466835b2228927d8dc1194cf8e6f52ba4b4cdb49cc954f31565d0c30fd44c8")).unwrap();
    /// # let expected = hex!("bc14fed2a5ae2c5c7e793f2a45f4f9aad84c7caa56139ee4a802806c5bb1a9cf4baa0e2df71bf3d0a943fbfb177afc1bd9c17995a6f409928548f3318d3f9b6300");
    /// # let keypair = Secp256k1KeyPair {
    /// #     public_key: PublicKey::from_secret_key(&secret_key),
    /// #     secret_key: Some(secret_key),
    /// # };
    /// let result = keypair.personal_sign(&sign_payload).unwrap();
    /// # assert_eq!(expected, result.as_slice())
    /// ```
    pub fn personal_sign(&self, message: &String) -> Result<Vec<u8>, Error> {
        let personal_message =
            format!("\x19Ethereum Signed Message:\n{}{}", message.len(), message);
        self.hashed_sign(&personal_message)
    }

    /// Signs `keccak256(message)`.
    /// Returns raw signature (r + s + v, 65-bytes).
    pub fn hashed_sign(&self, message: &String) -> Result<Vec<u8>, Error> {
        let hashed_message = super::util::hash_keccak256(&message);

        let (signature, recovery_id) =
            libsecp256k1::sign(&Message::parse(&hashed_message), &self.secret_key.unwrap());

        let mut result: Vec<u8> = vec![];
        result.extend_from_slice(&signature.r.b32());
        result.extend_from_slice(&signature.s.b32());
        result.extend_from_slice(&[recovery_id.serialize()]);
        if result.len() != 65 {
            return Err(Error::CryptoError(libsecp256k1::Error::InvalidInputLength));
        }
        Ok(result)
    }

    /// `web3.eth.personal.sign`, then `base64()` the result.
    /// # Examples
    ///
    /// ```rust
    /// # use kv_server::crypto::secp256k1::Secp256k1KeyPair;
    /// # use hex_literal::hex;
    /// # use libsecp256k1::{SecretKey, PublicKey};
    /// #
    /// let sign_payload = String::from("Test123!");
    /// # let secret_key = SecretKey::parse(&hex!("b5466835b2228927d8dc1194cf8e6f52ba4b4cdb49cc954f31565d0c30fd44c8")).unwrap();
    /// # let expected = format!("vBT+0qWuLFx+eT8qRfT5qthMfKpWE57kqAKAbFuxqc9Lqg4t9xvz0KlD+/sXevwb2cF5lab0CZKFSPMxjT+bYwA=");
    /// # let keypair = Secp256k1KeyPair {
    /// #     public_key: PublicKey::from_secret_key(&secret_key),
    /// #     secret_key: Some(secret_key),
    /// # };
    /// let result = keypair.base64_personal_sign(&sign_payload).unwrap();
    /// # assert_eq!(expected, result)
    /// ```
    pub fn base64_personal_sign(&self, message: &String) -> Result<String, Error> {
        let result = self.personal_sign(message)?;
        Ok(base64::encode(result))
    }

    /// Recover pubkey from an `eth_personalSign` signature with given plaintext message.
    /// # Examples
    ///
    /// ```rust
    /// # use kv_server::crypto::secp256k1::Secp256k1KeyPair;
    /// # use hex_literal::hex;
    /// # use libsecp256k1::{SecretKey, PublicKey, verify};
    /// #
    /// # let secret_key = SecretKey::parse(&hex!("b5466835b2228927d8dc1194cf8e6f52ba4b4cdb49cc954f31565d0c30fd44c8")).unwrap();
    /// # let public_key = PublicKey::from_secret_key(&secret_key);
    /// let sign_payload = String::from("Test123!");
    /// # let keypair = Secp256k1KeyPair {
    /// #   public_key,
    /// #   secret_key: Some(secret_key),
    /// # };
    /// # let signature = keypair.base64_personal_sign(&sign_payload).unwrap();
    /// # let sig = base64::decode(signature).unwrap();
    /// # println!("{:?}", sig);
    ///
    /// let recovered_pubkey = Secp256k1KeyPair::recover_from_personal_signature(&sig, &sign_payload).unwrap();
    /// assert_eq!(recovered_pubkey, public_key);
    /// ```
    pub fn recover_from_personal_signature(
        sig_r_s_recovery: &Vec<u8>,
        plain_payload: &str,
    ) -> Result<PublicKey, Error> {
        let personal_payload = format!(
            "\x19Ethereum Signed Message:\n{}{}",
            plain_payload.len(),
            plain_payload
        );
        let digest = hash_keccak256(&personal_payload);

        let mut recovery_id = sig_r_s_recovery
            .get(64)
            .ok_or_else(|| Error::CryptoError(libsecp256k1::Error::InvalidInputLength))?
            .clone();

        if recovery_id == 27 || recovery_id == 28 {
            recovery_id -= 27;
        }
        if recovery_id != 0 && recovery_id != 1 {
            return Err(Error::CryptoError(libsecp256k1::Error::InvalidSignature));
        }

        let signature = Signature::parse_standard_slice(&sig_r_s_recovery.as_slice()[..64])
            .map_err(|e| Error::from(e))?;
        let pubkey = libsecp256k1::recover(
            &Message::parse(&digest),
            &signature,
            &RecoveryId::parse(recovery_id).unwrap(),
        )?;

        Ok(pubkey)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::{
        crypto::util::hex_public_key,
        model::{
            self,
            kv_chains::{NewKVChain, SignPayload},
        },
        util,
    };

    use super::*;

    #[test]
    fn test_from_personal_signature() -> Result<(), Error> {
        let Secp256k1KeyPair {
            public_key,
            secret_key: _,
        } = Secp256k1KeyPair::from_pubkey_hex(
            "0x03e108f03e61a7e24dbd91a4eb621e3759fd1c2adb0fd6e3ec44e1a0f5bb45fa90",
        )?;
        let payload = SignPayload {
            version: "1".into(),
            uuid: uuid::Uuid::parse_str("fd042b27-0f21-476d-9e23-478c98ac6700")?,
            avatar: hex_public_key(&public_key),
            platform: "twitter".into(),
            identity: "weipingzhu2".into(),
            patch: json!({
                "com.mask.plugin": {
                    "twitter_weipingzhu2": {
                        "nickname": "vitalik.eth",
                        "userId": "WeipingZhu2",
                        "imageUrl": "https://pbs.twimg.com/profile_images/1514868277415084038/BJSpRyjq_normal.png",
                        "avatarId": "1514868277415084038",
                        "address": "0x495f947276749ce646f68ac8c248420045cb7b5e",
                        "tokenId": "84457744602723809043049191225279009657327463478214710277063869711841964851201"
                    }
                }
            }),
            created_at: 1650007736,
            previous: None,
        };
        let payload_string = serde_json::to_string(&payload)?;
        let signature = util::base64_to_vec("N1RKoa9le6dUdhCl+OO2FZuCWk20AdwMORCuyPqyn4kZW/+D+pvvYsuA6XQJwIzNmCwoj7eHuCF6mPds0fbmDwE=")?;
        let pubkey_recovered =
            Secp256k1KeyPair::recover_from_personal_signature(&signature, &payload_string)?;
        assert_eq!(public_key, pubkey_recovered);

        Ok(())
    }

    #[test]
    fn test_from_personal_signature_2() -> Result<(), Error> {
        let Secp256k1KeyPair {
            public_key,
            secret_key: _,
        } = Secp256k1KeyPair::from_pubkey_hex(
            "0x03e108f03e61a7e24dbd91a4eb621e3759fd1c2adb0fd6e3ec44e1a0f5bb45fa90",
        )?;
        let payload = SignPayload {
            version: "1".into(),
            uuid: uuid::Uuid::parse_str("b333f060-2cdd-4a7f-8fb1-c790c0fadc20")?,
            avatar: hex_public_key(&public_key),
            platform: "nextid".into(),
            identity: "0x03e108f03e61a7e24dbd91a4eb621e3759fd1c2adb0fd6e3ec44e1a0f5bb45fa90".into(),
            patch: json!({"com.maskbook.tip":[{"created_at":"1650188620","identity":"0x8c5494d05b4f18639834a0f1f4577d5c0a67adf0","invalid_reason":"","isDefault":0,"isPublic":1,"is_valid":true,"last_checked_at":"1650188620","platform":"ethereum"},{"created_at":"1650195158","identity":"0x2ec8ebb0a8eaa40e4ce620cf9f84a96df68d4669","invalid_reason":"","isDefault":1,"isPublic":1,"is_valid":true,"last_checked_at":"1650195158","platform":"ethereum"}]}),
            created_at: 1650209531,
            previous: None,
        };
        let expected_payload = r#"{"version":"1","uuid":"b333f060-2cdd-4a7f-8fb1-c790c0fadc20","avatar":"04e108f03e61a7e24dbd91a4eb621e3759fd1c2adb0fd6e3ec44e1a0f5bb45fa90d83378348df27416d1f6bf7c15f4220bfce331684ccefae5d07b9f4bab9fdb61","platform":"nextid","identity":"0x03e108f03e61a7e24dbd91a4eb621e3759fd1c2adb0fd6e3ec44e1a0f5bb45fa90","patch":{"com.maskbook.tip":[{"created_at":"1650188620","identity":"0x8c5494d05b4f18639834a0f1f4577d5c0a67adf0","invalid_reason":"","isDefault":0,"isPublic":1,"is_valid":true,"last_checked_at":"1650188620","platform":"ethereum"},{"created_at":"1650195158","identity":"0x2ec8ebb0a8eaa40e4ce620cf9f84a96df68d4669","invalid_reason":"","isDefault":1,"isPublic":1,"is_valid":true,"last_checked_at":"1650195158","platform":"ethereum"}]},"created_at":1650209531,"previous":null}"#;

        assert_eq!(
            serde_json::to_string(&payload)?,
            expected_payload.to_string()
        );
        let signature = util::base64_to_vec("VKtrENhPv8SpJF0By2jIy1/xP3KkUlla/AXEWyhjPiJv99FXa0yQDVvF8allspgfQWeh/5fzR6eRWPcNriC0zgE=")?;
        let pubkey_recovered =
            Secp256k1KeyPair::recover_from_personal_signature(&signature, &expected_payload)?;
        assert_eq!(public_key, pubkey_recovered);

        let mut conn = model::establish_connection();
        let mut new_kv = NewKVChain::for_persona(&mut conn, &public_key)?;
        new_kv.platform = payload.platform;
        new_kv.identity = payload.identity;
        new_kv.signature = signature;
        new_kv.patch = payload.patch.clone();
        new_kv.uuid = payload.uuid;
        new_kv.created_at = util::timestamp_to_naive(payload.created_at);

        assert_eq!(
            expected_payload,
            serde_json::to_string(&new_kv.generate_signature_payload()?)?
        );

        Ok(())
    }
}
