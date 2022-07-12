use crate::{
    controller::{query_parse, Request, Response},
    crypto::{secp256k1::Secp256k1KeyPair, util::hex_public_key},
    error::Error,
    model::{establish_connection, kv},
};
use diesel::PgConnection;
use http::StatusCode;
use libsecp256k1::PublicKey;
use serde::{Deserialize, Serialize};

use super::json_response;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResponse {
    pub persona: String,
    pub avatar: String,
    pub proofs: Vec<QueryResponseSingleProof>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResponseSingleProof {
    pub platform: String,
    pub identity: String,
    pub content: serde_json::Value,
}

pub async fn controller(req: Request) -> Result<Response, Error> {
    let params = query_parse(req);
    let avatar_hex = params
        .get("avatar")
        .or(params.get("persona"))
        .ok_or(Error::ParamMissing("avatar".into()))?;
    let Secp256k1KeyPair {
        public_key,
        secret_key: _,
    } = Secp256k1KeyPair::from_pubkey_hex(avatar_hex)?;

    let conn = establish_connection();
    let response = query_response(&conn, &public_key)?;

    json_response(StatusCode::OK, &response)
}

pub fn query_response(
    conn: &PgConnection,
    persona_public_key: &PublicKey,
) -> Result<QueryResponse, Error> {
    let results = kv::find_all_by_persona(&conn, persona_public_key)?;

    let persona_hex = hex_public_key(persona_public_key);
    let mut response = QueryResponse {
        persona: format!("0x{}", persona_hex),
        avatar: format!("0x{}", persona_hex),
        proofs: vec![],
    };
    for proof in results.iter() {
        let proof_single = QueryResponseSingleProof {
            platform: proof.platform.clone(),
            identity: proof.identity.clone(),
            content: proof.content.clone(),
        };
        response.proofs.push(proof_single);
    }
    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::{secp256k1::Secp256k1KeyPair, util::hex_public_key};
    use fake::Fake;
    use http::Method;
    use serde_json::json;

    #[tokio::test]
    async fn test_controller_smoke() {
        let Secp256k1KeyPair {
            public_key,
            secret_key: _,
        } = Secp256k1KeyPair::generate();
        let pubkey_hex = hex_public_key(&public_key);
        let req: Request = ::http::Request::builder()
            .method(Method::GET)
            .uri(format!("http://localhost/test?persona={}", pubkey_hex))
            .body("".into())
            .unwrap();

        let resp = controller(req).await.unwrap();
        let body: QueryResponse = serde_json::from_str(resp.body()).unwrap();
        assert_eq!(0, body.proofs.len());
        assert_eq!(format!("0x{}", pubkey_hex), body.avatar);
    }

    #[tokio::test]
    async fn test_controller_with_result() {
        let conn = establish_connection();
        let Secp256k1KeyPair {
            public_key,
            secret_key: _,
        } = Secp256k1KeyPair::generate();
        kv::find_or_create(&conn, "twitter", &fake::Faker.fake::<String>(), &public_key).unwrap();

        let req: Request = ::http::Request::builder()
            .method(Method::GET)
            .uri(format!(
                "http://localhost/test?persona=0x{}",
                hex_public_key(&public_key)
            ))
            .body("".into())
            .unwrap();
        let resp = controller(req).await.unwrap();
        let body: QueryResponse = serde_json::from_str(resp.body()).unwrap();
        assert_eq!(1, body.proofs.len());
        assert_eq!(format!("0x{}", hex_public_key(&public_key)), body.avatar);
        assert_eq!("twitter", body.proofs.first().unwrap().platform);
        assert_eq!(json!({}), body.proofs.first().unwrap().content);
    }
}
