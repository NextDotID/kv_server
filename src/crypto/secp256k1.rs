use crate::{crypto::util::hash_keccak256, error::Error};
use libsecp256k1::{Message, PublicKey, RecoveryId, SecretKey, Signature};
use rand::rngs;

pub struct Secp256k1KeyPair {
    pub public_key: PublicKey,
    pub secret_key: SecretKey,
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
    /// # assert_eq!(32, keypair.secret_key.serialize().len());
    /// ```
    pub fn generate() -> Self {
        let mut rng = rngs::OsRng;
        let secret_key = SecretKey::random(&mut rng);
        let public_key = PublicKey::from_secret_key(&secret_key);

        Self {
            public_key,
            secret_key,
        }
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
    /// #     secret_key,
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
            libsecp256k1::sign(&Message::parse(&hashed_message), &self.secret_key);

        let mut result: Vec<u8> = vec![];
        result.extend_from_slice(&signature.r.b32());
        result.extend_from_slice(&signature.s.b32());
        result.extend_from_slice(&[recovery_id.serialize()]);
        if result.len() != 65 {
            return Err(Error::CryptoError {
                source: libsecp256k1::Error::InvalidInputLength,
            });
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
    /// #     secret_key,
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
    /// #   secret_key,
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

        let recovery_id = sig_r_s_recovery.get(64).ok_or_else(|| Error::CryptoError {
            source: libsecp256k1::Error::InvalidInputLength,
        })?;
        let signature = Signature::parse_standard_slice(&sig_r_s_recovery.as_slice()[..64])
            .map_err(|e| Error::from(e))?;
        let pubkey = libsecp256k1::recover(
            &Message::parse(&digest),
            &signature,
            &RecoveryId::parse(*recovery_id).unwrap(),
        )?;

        Ok(pubkey)
    }
}
