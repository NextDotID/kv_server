mod tests;

use crate::{crypto::secp256k1::Secp256k1KeyPair, error::Error, types::subkey::Algorithm};
use http::{Response, StatusCode};
use hyper::{body::HttpBody as _, client::HttpConnector, Body, Client};
use hyper_tls::HttpsConnector;
use libsecp256k1::PublicKey;
use serde::Deserialize;

/// https://github.com/nextdotid/proof-server/blob/master/docs/api.apib
#[derive(Deserialize, Debug)]
pub struct ProofQueryResponse {
    pub pagination: ProofQueryResponsePagination,
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
pub struct ProofQueryResponsePagination {
    pub total: u32,
    pub per: u32,
    pub current: u32,
    pub next: u32,
}

#[derive(Deserialize, Debug)]
pub struct SubkeyQueryResponse {
    pub subkeys: Vec<SubkeyQueryResponseSingle>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct SubkeyQueryResponseSingle {
    pub avatar: String,
    pub algorithm: Algorithm,
    pub public_key: String,
    pub name: String,
    #[serde(rename = "RP_ID")]
    pub rp_id: String,
    pub created_at: u32,
}

#[derive(Deserialize, Debug)]
pub struct ErrorResponse {
    pub message: String,
}

fn make_client() -> Client<HttpsConnector<HttpConnector>> {
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
async fn query_avatar(base: &str, persona: &str) -> Result<ProofQueryResponse, Error> {
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

async fn query_subkey(
    base: &str,
    algorithm: &Algorithm,
    public_key: &str,
) -> Result<SubkeyQueryResponse, Error> {
    let client = make_client();
    let uri = format!(
        "{}/v1/subkey?algorithm={}&public_key={}",
        base,
        algorithm.to_string(),
        public_key
    )
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

    parse_body(&mut resp).await.map_err(|e| e.into())
}

/// Determine if given subkey exists on ProofService.  Returns the binded avatar public key.
pub async fn find_subkey(
    algorithm: &Algorithm,
    public_key: &str,
) -> Result<SubkeyQueryResponseSingle, Error> {
    let query_result =
        query_subkey(&crate::config::C.proof_service.url, &algorithm, public_key).await?;
    if query_result.subkeys.len() == 0 {
        return Err(Error::General(
            "Subkey not found on ProofService".into(),
            StatusCode::BAD_REQUEST,
        ));
    };
    let subkey_found = query_result
        .subkeys
        .iter()
        .find(|&sk| sk.public_key == public_key && sk.algorithm == *algorithm)
        .ok_or(Error::General(
            "Subkey not found on ProofService".into(),
            StatusCode::BAD_REQUEST,
        ))?;
    Ok(subkey_found.clone())
}

/// Determine if persona-platform-identity pair can set a KV.
pub async fn can_set_kv(
    persona_pubkey: &PublicKey,
    platform: &str,
    identity: &str,
) -> Result<(), Error> {
    // FIXME: super stupid test stub
    if cfg!(test) {
        return Ok(());
    }
    // KV of NextID: validate if identity == persona.
    if platform == "nextid" {
        let Secp256k1KeyPair {
            public_key: identity_pubkey,
            secret_key: _,
        } = Secp256k1KeyPair::from_pubkey_hex(identity)?;
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
    let persona_compressed_hex =
        format!("0x{}", hex::encode(persona_pubkey.serialize_compressed()));
    let query_response =
        query_avatar(&crate::config::C.proof_service.url, &persona_compressed_hex).await?;
    if query_response.ids.len() == 0 {
        return Err(Error::General(
            format!(
                "Persona not found on ProofService: {}",
                persona_compressed_hex
            ),
            StatusCode::BAD_REQUEST,
        ));
    }

    // FIXME: maybe no need to iter like this.
    let persona_found = query_response
        .ids
        .iter()
        .find(|id| id.persona == persona_compressed_hex)
        .ok_or_else(|| {
            Error::General(
                format!(
                    "Persona not found on ProofService: {}",
                    persona_compressed_hex
                ),
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
