use crate::{
    controller::{Request, Response, json_parse_body, json_response},
    crypto::secp256k1::Secp256k1KeyPair,
    error::Error,
    model::{establish_connection, kv_chains::NewKVChain}, proof_client::can_set_kv,
};
use http::StatusCode;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PayloadRequest {
    pub persona: String,
    pub platform: String,
    pub identity: String,
    pub patch: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PayloadResponse {
    pub uuid: String,
    pub sign_payload: String,
}

pub async fn controller(req: Request) -> Result<Response, Error> {
    let params: PayloadRequest = json_parse_body(&req)?;
    let keypair = Secp256k1KeyPair::from_pubkey_hex(&params.persona)?;
    can_set_kv(&keypair.public_key, &params.platform, &params.identity).await?;
    let conn = establish_connection();
    let mut new_kvchain = NewKVChain::for_persona(&conn, &keypair.public_key)?;

    new_kvchain.platform = params.platform;
    new_kvchain.identity = params.identity;
    new_kvchain.patch = params.patch;
    let sign_payload = new_kvchain.sign_body()?;

    Ok(json_response(StatusCode::OK, &PayloadResponse{
        sign_payload,
        uuid: new_kvchain.uuid.to_string(),
    })?)
}

#[cfg(test)]
mod tests {
    use diesel::{PgConnection, insert_into, RunQueryDsl};
    use fake::{Faker, Fake};
    use http::Method;
    use libsecp256k1::PublicKey;
    use serde_json::json;

    use crate::{crypto::util::compress_public_key, model::kv_chains::KVChain, schema::kv_chains::dsl::*, util::vec_to_base64};

    use super::*;

    fn generate_data(conn: &PgConnection, persona_pubkey: &PublicKey) -> Result<KVChain, Error> {
        let new_uuid = ::uuid::Uuid::new_v4();
        let persona_bytes = persona_pubkey.serialize().to_vec();
        let new_platform: String = Faker.fake();
        let new_identity: String = Faker.fake();
        insert_into(kv_chains)
            .values(&NewKVChain {
                uuid: new_uuid,
                persona: persona_bytes,
                platform: new_platform,
                identity: new_identity,
                patch: json!({ "test": "abc" }),
                previous_id: None,
                signature: vec![1],
            })
            .get_result(conn)
            .map_err(|e| e.into())
    }

    #[tokio::test]
    async fn test_success() {
        let Secp256k1KeyPair { public_key, secret_key: _ } = Secp256k1KeyPair::generate();

        let req_body = PayloadRequest{
            persona: compress_public_key(&public_key),
            platform: "facebook".into(),
            identity: Faker.fake(),
            patch: json!({"test":"abc"}),
        };
        let req: Request = ::http::Request::builder()
            .method(Method::POST)
            .uri(format!("http://localhost?test")).
            body(serde_json::to_string(&req_body).unwrap())
            .unwrap();
        let resp = controller(req).await.unwrap();
        let body: PayloadResponse = serde_json::from_str(resp.body()).unwrap();
        assert!(body.uuid.len() > 0);
        let payload = body.sign_payload;
        assert!(payload.contains(&compress_public_key(&public_key)));
        assert!(payload.contains(r#""test":"abc""#));
        assert!(payload.contains("facebook"));
        assert!(payload.contains(&req_body.identity));
        assert!(payload.contains(r#""previous":null"#));
    }

    #[tokio::test]
    async fn test_with_previous() {
        let conn = establish_connection();
        let Secp256k1KeyPair { public_key, secret_key: _ } = Secp256k1KeyPair::generate();
        let old_kv_chain = generate_data(&conn, &public_key).unwrap();

        let req_body = PayloadRequest{
            persona: compress_public_key(&public_key),
            platform: "facebook".into(),
            identity: Faker.fake(),
            patch: json!({"test":"abc"}),
        };
        let req: Request = ::http::Request::builder()
            .method(Method::POST)
            .uri(format!("http://localhost?test")).
            body(serde_json::to_string(&req_body).unwrap())
            .unwrap();
        let resp = controller(req).await.unwrap();
        let body: PayloadResponse = serde_json::from_str(resp.body()).unwrap();
        let payload = body.sign_payload;
        assert!(payload.contains(&vec_to_base64(&old_kv_chain.signature)));
    }
}
