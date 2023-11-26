use crate::{
    controller::{json_parse_body, json_response, Request, Response},
    crypto::secp256k1::Secp256k1KeyPair,
    error::Error,
    model::{establish_connection, kv_chains::NewKVChain},
    proof_client::{can_set_kv, self},
};
use http::StatusCode;
use serde::{Deserialize, Serialize};

use super::error_response;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PayloadRequest {
    pub avatar: Option<String>,
    pub algorithm: Option<crate::types::subkey::Algorithm>,
    pub public_key: Option<String>,
    pub platform: String,
    pub identity: String,
    pub patch: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PayloadResponse {
    pub uuid: String,
    pub sign_payload: String,
    pub created_at: i64,
}

pub async fn controller(req: Request) -> Result<Response, Error> {
    let params: PayloadRequest = json_parse_body(&req)?;
    if params.avatar.is_some() {
        sign_payload_with_avatar(&params).await
    } else if params.algorithm.is_some() && params.public_key.is_some() {
        sign_payload_with_subkey(&params).await
    } else {
        Ok(error_response(
            Error::ParamError("(avatar) or (algorithm, public_key) is not provided".into())
        ))
    }
}

async fn sign_payload_with_avatar(params: &PayloadRequest) -> Result<Response, Error> {
    let keypair = Secp256k1KeyPair::from_pubkey_hex(params.avatar.as_ref().unwrap())?;
    can_set_kv(&keypair.public_key, &params.platform, &params.identity).await?;
    let mut conn = establish_connection();
    let mut new_kvchain = NewKVChain::for_persona(&mut conn, &keypair.public_key)?;

    new_kvchain.platform = params.platform.clone();
    new_kvchain.identity = params.identity.clone();
    new_kvchain.patch = params.patch.clone();
    let sign_payload = new_kvchain.generate_signature_payload()?;

    Ok(json_response(
        StatusCode::OK,
        &PayloadResponse {
            sign_payload: serde_json::to_string(&sign_payload)?,
            uuid: sign_payload.uuid.to_string(),
            created_at: sign_payload.created_at,
        },
    )?)
}

async fn sign_payload_with_subkey(params: &PayloadRequest) -> Result<Response, Error> {
    let algorithm = params.algorithm.as_ref().unwrap();
    let public_key = params.public_key.as_ref().unwrap();
    let subkey = proof_client::find_subkey(&algorithm, &public_key).await?;
    let avatar = Secp256k1KeyPair::from_pubkey_hex(&subkey.avatar)?;
    can_set_kv(&avatar.public_key, &params.platform, &params.identity).await?;

    let mut conn = establish_connection();
    let mut new_kvchain = NewKVChain::for_persona(&mut conn, &avatar.public_key)?;
    new_kvchain.platform = params.platform.clone();
    new_kvchain.identity = params.identity.clone();
    new_kvchain.patch = params.patch.clone();
    let sign_payload = new_kvchain.generate_signature_payload()?;

    Ok(json_response(
        StatusCode::OK,
        &PayloadResponse {
            sign_payload: serde_json::to_string(&sign_payload)?,
            uuid: sign_payload.uuid.to_string(),
            created_at: sign_payload.created_at,
        },
    )?)
}

#[cfg(test)]
mod tests {
    use diesel::{insert_into, PgConnection, RunQueryDsl};
    use fake::{Fake, Faker};
    use http::Method;
    use libsecp256k1::PublicKey;
    use serde_json::json;

    use crate::{
        crypto::util::{compress_public_key, hex_public_key},
        model::kv_chains::KVChain,
        schema::kv_chains::dsl::*,
        util::{naive_now, vec_to_base64},
    };

    use super::*;

    fn generate_data(conn: &mut PgConnection, persona_pubkey: &PublicKey) -> Result<KVChain, Error> {
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
                signature_payload: "".into(),
                created_at: naive_now(),
                arweave_id: None,
            })
            .get_result(conn)
            .map_err(|e| e.into())
    }

    #[tokio::test]
    async fn test_success() {
        let Secp256k1KeyPair {
            public_key,
            secret_key: _,
        } = Secp256k1KeyPair::generate();

        let req_body = PayloadRequest {
            avatar: Some(compress_public_key(&public_key)),
            algorithm: None,
            public_key: None,
            platform: "facebook".into(),
            identity: Faker.fake(),
            patch: json!({"test":"abc"}),
        };
        let req: Request = ::http::Request::builder()
            .method(Method::POST)
            .uri(format!("http://localhost?test"))
            .body(serde_json::to_string(&req_body).unwrap())
            .unwrap();
        let resp = controller(req).await.unwrap();
        let body: PayloadResponse = serde_json::from_str(resp.body()).unwrap();
        assert!(body.uuid.len() > 0);
        let payload = body.sign_payload;
        assert!(payload.contains(&hex_public_key(&public_key)));
        assert!(payload.contains(r#""test":"abc""#));
        assert!(payload.contains("facebook"));
        assert!(payload.contains(&req_body.identity));
        assert!(payload.contains(r#""previous":null"#));
    }

    #[tokio::test]
    async fn test_with_previous() {
        let mut conn = establish_connection();
        let Secp256k1KeyPair {
            public_key,
            secret_key: _,
        } = Secp256k1KeyPair::generate();
        let old_kv_chain = generate_data(&mut conn, &public_key).unwrap();

        let req_body = PayloadRequest {
            avatar: Some(compress_public_key(&public_key)),
            algorithm: None,
            public_key: None,
            platform: "facebook".into(),
            identity: Faker.fake(),
            patch: json!({"test":"abc"}),
        };
        let req: Request = ::http::Request::builder()
            .method(Method::POST)
            .uri(format!("http://localhost?test"))
            .body(serde_json::to_string(&req_body).unwrap())
            .unwrap();
        let resp = controller(req).await.unwrap();
        let body: PayloadResponse = serde_json::from_str(resp.body()).unwrap();
        let payload = body.sign_payload;
        assert!(payload.contains(&vec_to_base64(&old_kv_chain.signature)));
    }
}
