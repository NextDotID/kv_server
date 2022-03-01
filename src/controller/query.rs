use crate::{
    controller::{query_parse, Request, Response},
    error::Error,
    model::{establish_connection, kv},
};
use http::StatusCode;
use serde::{Deserialize, Serialize};

use super::json_response;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct QueryResponse<'a> {
    pub persona: &'a str,
    pub proofs: Vec<QueryResponseSingleProof<'a>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct QueryResponseSingleProof<'a> {
    pub platform: &'a str,
    pub identity: &'a str,
    pub content: serde_json::Value,
}

pub async fn controller(req: Request) -> Result<Response, Error> {
    let params = query_parse(req);
    let persona_hex = params
        .get("persona")
        .ok_or(Error::ParamMissing("persona".into()))?;

    let conn = establish_connection();
    let results = kv::find_all_by_persona(&conn, persona_hex)?;

    let mut response = QueryResponse {
        persona: persona_hex,
        proofs: vec![],
    };
    for proof in results.iter() {
        let proof_single = QueryResponseSingleProof {
            platform: proof.platform.as_str(),
            identity: proof.identity.as_str(),
            content: proof.content.clone(),
        };
        response.proofs.push(proof_single);
    }

    json_response(StatusCode::OK, &response)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::{secp256k1::Secp256k1KeyPair, util::compress_public_key};
    use fake::Fake;
    use http::Method;
    use serde_json::json;

    #[tokio::test]
    async fn test_controller_smoke() {
        let Secp256k1KeyPair {
            public_key,
            secret_key: _,
        } = Secp256k1KeyPair::generate();
        let pubkey_hex = compress_public_key(&public_key);
        let req: Request = ::http::Request::builder()
            .method(Method::GET)
            .uri(format!("http://localhost/test?persona={}", pubkey_hex))
            .body("".into())
            .unwrap();

        let resp = controller(req).await.unwrap();
        let body: QueryResponse = serde_json::from_str(resp.body()).unwrap();
        assert_eq!(0, body.proofs.len());
        assert_eq!(pubkey_hex, body.persona);
    }

    #[tokio::test]
    async fn test_controller_with_result() {
        let conn = establish_connection();
        let Secp256k1KeyPair {
            public_key,
            secret_key: _,
        } = Secp256k1KeyPair::generate();
        let pubkey_hex = compress_public_key(&public_key);
        kv::find_or_create(&conn, "twitter", &fake::Faker.fake::<String>(), &pubkey_hex).unwrap();

        let req: Request = ::http::Request::builder()
            .method(Method::GET)
            .uri(format!("http://localhost/test?persona={}", pubkey_hex))
            .body("".into())
            .unwrap();
        let resp = controller(req).await.unwrap();
        let body: QueryResponse = serde_json::from_str(resp.body()).unwrap();
        assert_eq!(1, body.proofs.len());
        assert_eq!(pubkey_hex, body.persona);
        assert_eq!("twitter", body.proofs.first().unwrap().platform);
        assert_eq!(json!({}), body.proofs.first().unwrap().content);
    }
}
