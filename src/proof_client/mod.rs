mod tests;

use crate::{crypto::secp256k1::Secp256k1KeyPair, error::Error};
use http::{Response, StatusCode};
use hyper::{body::HttpBody as _, client::HttpConnector, Body, Client};
use hyper_tls::HttpsConnector;
use libsecp256k1::PublicKey;
use serde::Deserialize;

/// https://github.com/nextdotid/proof-server/blob/master/docs/api.apib
#[derive(Deserialize, Debug)]
pub struct ProofQueryResponse {
    pub ids: Vec<ProofPersona>,
}

#[derive(Deserialize, Debug)]
pub struct ProofPersona {
    pub persona: String,
    pub proofs: Vec<Proof>,
}

#[derive(Deserialize, Debug)]
pub struct Proof {
    pub platform: String,
    pub identity: String,
    pub created_at: String,
    pub last_checked_at: String,
    pub is_valid: bool,
    pub invalid_reason: String,
}

#[derive(Deserialize, Debug)]
pub struct ErrorResponse {
    pub message: String,
}

pub fn make_client() -> Client<HttpsConnector<HttpConnector>> {
    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(https);
    client
}

async fn parse_body<T>(resp: &mut Response<Body>) -> Result<T, Error>
where
    T: for<'de> Deserialize<'de>,
{
    let mut body_bytes: Vec<u8> = vec![];
    while let Some(chunk) = resp.body_mut().data().await {
        let mut chunk_bytes = chunk.unwrap().to_vec();
        body_bytes.append(&mut chunk_bytes);
    }
    let body = std::str::from_utf8(&body_bytes).unwrap();

    Ok(serde_json::from_str(&body)?)
}

/// Persona should be 33-bytes hexstring (`0x[0-9a-f]{66}`)
pub async fn query(base: &str, persona: &str) -> Result<ProofQueryResponse, Error> {
    let client = make_client();
    let uri = format!("{}/v1/proof?platform=nextid&identity={}", base, persona)
        .parse()
        .unwrap();
    let mut resp = client.get(uri).await?;
    if !resp.status().is_success() {
        let body: ErrorResponse = parse_body(&mut resp).await?;
        return Err(Error::General(
            format!("ProofService error: {}", body.message),
            resp.status(),
        ));
    }
    let body: ProofQueryResponse = parse_body(&mut resp).await?;
    Ok(body)
}

/// Determine if persona-platform-identity pair can set a KV.
pub async fn can_set_kv(
    persona_pubkey: &PublicKey,
    platform: &String,
    identity: &String,
) -> Result<(), Error> {
    // FIXME: super stupid test stub
    if cfg!(test) {
        return Ok(());
    }
    // KV of NextID: validate if identity == persona.
    if *platform == "nextid".to_string() {
        let Secp256k1KeyPair {
            public_key: identity_pubkey,
            secret_key: _,
        } = Secp256k1KeyPair::from_pubkey_hex(&identity)?;
        if identity_pubkey == *persona_pubkey {
            return Ok(());
        } else {
            return Err(Error::General(
                format!("Identity and persona not match when 'platform' is 'nextid' ."),
                StatusCode::BAD_REQUEST,
            ));
        }
    }
    // Else: connect to ProofService
    let persona_full_hex = format!("0x{}", hex::encode(persona_pubkey.serialize()));
    let query_response = query(&crate::config::C.proof_service.url, &persona_full_hex).await?;
    if query_response.ids.len() == 0 {
        return Err(Error::General(
            format!(
                "Persona not found found on ProofService: {}",
                persona_full_hex
            ),
            StatusCode::BAD_REQUEST,
        ));
    }

    // FIXME: maybe no need to iter like this.
    let persona_found = query_response
        .ids
        .iter()
        .find(|id| id.persona == persona_full_hex)
        .ok_or_else(|| {
            Error::General(
                format!("Persona not found on ProofService: {}", persona_full_hex),
                StatusCode::BAD_REQUEST,
            )
        })?;

    let _proof_found = persona_found
        .proofs
        .iter()
        .find(|proof| proof.platform == *platform && proof.identity == *identity)
        .ok_or_else(|| {
            Error::General(
                format!("Proof not found under this persona."),
                StatusCode::BAD_REQUEST,
            )
        })?;

    Ok(())
}
