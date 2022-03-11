use super::{json_response, query::query_response};
use crate::{
    controller::{json_parse_body, Request, Response},
    crypto::secp256k1::Secp256k1KeyPair,
    error::Error,
    model::{self, kv_chains::NewKVChain},
    proof_client::can_set_kv,
    util::{base64_to_vec, timestamp_to_naive},
};
use http::StatusCode;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
struct UploadRequest {
    pub persona: String,
    pub platform: String,
    pub identity: String,
    pub signature: String,
    pub uuid: String,
    pub created_at: i64,
    pub patch: serde_json::Value,
}

pub async fn controller(request: Request) -> Result<Response, Error> {
    let req: UploadRequest = json_parse_body(&request)?;
    let sig = base64_to_vec(&req.signature)?;
    let persona = Secp256k1KeyPair::from_pubkey_hex(&req.persona)?;
    let uuid = uuid::Uuid::parse_str(&req.uuid)?;
    can_set_kv(&persona.public_key, &req.platform, &req.identity).await?;

    let conn = model::establish_connection();
    let mut new_kv = NewKVChain::for_persona(&conn, &persona.public_key)?;
    new_kv.platform = req.platform;
    new_kv.identity = req.identity;
    new_kv.signature = sig;
    new_kv.patch = req.patch.clone();
    new_kv.uuid = uuid;
    new_kv.created_at = timestamp_to_naive(req.created_at);
    new_kv.signature_payload = serde_json::to_string(&new_kv.generate_signature_payload()?).unwrap();

    // Validate signature
    new_kv.validate()?;

    // Valid. Insert it.
    let kv_link = new_kv.finalize(&conn)?;
    // Apply patch
    kv_link.perform_patch(&conn)?;

    // All done. Build response.
    let response = query_response(&conn, &persona.public_key)?;
    json_response(StatusCode::CREATED, &response)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        controller::query::QueryResponse,
        crypto::util::{compress_public_key, hex_public_key},
        model::{establish_connection, kv},
        util::{vec_to_base64, naive_now},
    };
    use fake::{Fake, Faker};
    use http::Method;
    use serde_json::json;

    #[tokio::test]
    async fn test_newly_create() {
        let keypair = Secp256k1KeyPair::generate();
        let mut new_kv_chain = NewKVChain {
            uuid: uuid::Uuid::new_v4(),
            persona: keypair.public_key.serialize().to_vec(),
            platform: Faker.fake(),
            identity: Faker.fake(),
            patch: json!({"test": "abc"}),
            previous_id: None,
            signature: vec![],
            signature_payload: "".into(),
            created_at: naive_now(),
        };
        new_kv_chain.signature = new_kv_chain.sign(&keypair).unwrap();

        let req_body = UploadRequest {
            persona: compress_public_key(&keypair.public_key),
            platform: new_kv_chain.platform.clone(),
            identity: new_kv_chain.identity,
            signature: vec_to_base64(&new_kv_chain.signature),
            uuid: new_kv_chain.uuid.to_string(),
            patch: new_kv_chain.patch.clone(),
            created_at: new_kv_chain.created_at.timestamp(),
        };
        let req: Request = ::http::Request::builder()
            .method(Method::POST)
            .uri(format!("http://localhost/test"))
            .body(serde_json::to_string(&req_body).unwrap())
            .unwrap();

        let resp = controller(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);
        let resp_body: QueryResponse = serde_json::from_str(resp.body()).unwrap();
        assert_eq!(1, resp_body.proofs.len());
        assert_eq!(format!("0x{}", hex_public_key(&keypair.public_key)), resp_body.persona);
        assert_eq!(
            new_kv_chain.platform,
            resp_body.proofs.first().unwrap().platform
        );
        assert_eq!(
            new_kv_chain.patch,
            resp_body.proofs.first().unwrap().content
        );
    }

    #[tokio::test]
    async fn test_modify_existed() {
        let keypair = Secp256k1KeyPair::generate();
        let conn = establish_connection();
        let platform: String = Faker.fake();
        let identity: String = Faker.fake();
        let (existed_kv, _) =
            kv::find_or_create(&conn, &platform, &identity, &keypair.public_key).unwrap();
        existed_kv
            .patch(&conn, &json!({"test": "existed"}))
            .unwrap();

        let mut new_kv_chain = NewKVChain {
            uuid: uuid::Uuid::new_v4(),
            persona: keypair.public_key.serialize().to_vec(),
            platform: platform.clone(),
            identity: identity.clone(),
            patch: json!({"test": null, "test2": "new kv"}),
            previous_id: None,
            signature: vec![],
            signature_payload: "".into(),
            created_at: naive_now(),
        };
        let sig = new_kv_chain.sign(&keypair).unwrap();
        new_kv_chain.signature = sig;

        let req_body = UploadRequest {
            persona: compress_public_key(&keypair.public_key),
            platform: new_kv_chain.platform.clone(),
            identity: new_kv_chain.identity,
            signature: vec_to_base64(&new_kv_chain.signature),
            uuid: new_kv_chain.uuid.to_string(),
            patch: new_kv_chain.patch.clone(),
            created_at: new_kv_chain.created_at.timestamp(),
        };
        let req: Request = ::http::Request::builder()
            .method(Method::POST)
            .uri(format!("http://localhost/test"))
            .body(serde_json::to_string(&req_body).unwrap())
            .unwrap();

        let resp = controller(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);
        let resp_body: QueryResponse = serde_json::from_str(resp.body()).unwrap();
        assert_eq!(1, resp_body.proofs.len());
        let proof = resp_body.proofs.first().unwrap();
        assert_eq!(proof.content, json!({"test2": "new kv"}));
    }
}
