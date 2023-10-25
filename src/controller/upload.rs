use super::{json_response, query::query_response};
use crate::{
    controller::{json_parse_body, Request, Response},
    crypto::secp256k1::Secp256k1KeyPair,
    error::Error,
    model::{self, kv_chains::NewKVChain, arweave::KVChainArweaveDocument},
    proof_client::can_set_kv,
    util::{base64_to_vec, timestamp_to_naive},
};
use http::StatusCode;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
struct UploadRequest {
    pub persona: Option<String>,
    pub avatar: Option<String>,
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
    let avatar = req.avatar.clone();
    let persona = Secp256k1KeyPair::from_pubkey_hex(
        &req.avatar
            .or(req.persona)
            .ok_or_else(|| Error::ParamError("avatar not found".into()))?,
    )?;
    let uuid = uuid::Uuid::parse_str(&req.uuid)?;
    can_set_kv(&persona.public_key, &req.platform, &req.identity).await?;

    let mut conn = model::establish_connection();
    let mut new_kv = NewKVChain::for_persona(&mut conn, &persona.public_key)?;
    new_kv.platform = req.platform;
    new_kv.identity = req.identity;
    new_kv.signature = sig;
    new_kv.patch = req.patch.clone();
    new_kv.uuid = uuid;
    new_kv.created_at = timestamp_to_naive(req.created_at);
    new_kv.signature_payload =
        serde_json::to_string(&new_kv.generate_signature_payload()?).unwrap();

    // Validate signature
    new_kv.validate()?;

    let previous_arweave_id = new_kv.clone().find_last_chain_arweave(&mut conn)?;

    // Try take the kvchain data upload to the arweave.
    let arweave_document = KVChainArweaveDocument{
        avatar: avatar.unwrap_or("".into()),
        uuid,
        persona: vec![],
        platform: new_kv.platform.clone(),
        identity: new_kv.identity.clone(),
        patch: new_kv.patch.clone(),
        signature: new_kv.signature.clone(),
        created_at: new_kv.created_at,
        signature_payload: new_kv.signature_payload.clone(),
        previous_id: new_kv.previous_id.clone(),
        previous_arweave_id: previous_arweave_id.clone(),
    };

    // Valid. Insert it.
    let kv_link = new_kv.finalize(&mut conn)?;

    // Apply patch
    kv_link.perform_patch(&mut conn)?;

    // // Upload to arweave
    // tokio::spawn(async move {
    //     let result = arweave_document.upload_to_arweave().await.ok();
    //     let _ = kv_link.insert_arweave_id(&mut conn, result);
    // });

    let result = arweave_document.upload_to_arweave().await.ok();
    let _ = kv_link.insert_arweave_id(&mut conn, result);
    
    // All done. Build response.
    let response = query_response(&mut conn, &persona.public_key)?;
    
    json_response(StatusCode::CREATED, &response)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        controller::query::QueryResponse,
        crypto::util::{compress_public_key, hex_public_key},
        model::{establish_connection, kv, kv_chains::{find_all_by_identity, KVChain}},
        util::{naive_now, vec_to_base64},
    };
    use fake::{Fake, Faker};
    use http::Method;
    use libsecp256k1::PublicKey;
    use serde_json::{json, Value};

    /// Create a request and send it to controller.
    /// And then return the response body.
    async fn create_req_and_send(new_kv_chain: NewKVChain, public_key: PublicKey) -> QueryResponse {
        let req_body = UploadRequest {
            persona: Some(compress_public_key(&public_key)),
            avatar: Some(compress_public_key(&public_key)),
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

        serde_json::from_str(resp.body()).unwrap()
    }
    
    fn create_new_kv_chain(persona: PublicKey, platform: &String, identity: &String, patch: Value) -> NewKVChain {
        NewKVChain {
            uuid: uuid::Uuid::new_v4(),
            persona: persona.serialize().to_vec(),
            platform: platform.clone(),
            identity: identity.clone(),
            patch,
            previous_id: None,
            signature: vec![],
            signature_payload: "".into(),
            created_at: naive_now(),
            arweave_id: None,
        }
    }

    #[tokio::test]
    async fn test_newly_create() {
        let keypair = Secp256k1KeyPair::generate();
        let mut new_kv_chain = create_new_kv_chain(
            keypair.public_key, &Faker.fake(), &Faker.fake(), json!({"test": "abc"}));
        new_kv_chain.signature = new_kv_chain.sign(&keypair).unwrap();

        let resp_body = create_req_and_send(new_kv_chain.clone(), keypair.public_key).await;
        assert_eq!(1, resp_body.proofs.len());
        assert_eq!(
            format!("0x{}", hex_public_key(&keypair.public_key)),
            resp_body.persona
        );
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
        let mut conn = establish_connection();
        let platform: String = Faker.fake();
        let identity: String = Faker.fake();
        let (existed_kv, _) =
            kv::find_or_create(&mut conn, &platform, &identity, &keypair.public_key).unwrap();
        existed_kv
            .patch(&mut conn, &json!({"test": "existed"}))
            .unwrap();

        let mut new_kv_chain = create_new_kv_chain(
            keypair.public_key, &platform, &identity, json!({"test": null, "test2": "new kv"}));
        new_kv_chain.signature = new_kv_chain.sign(&keypair).unwrap();

        let resp_body = create_req_and_send(new_kv_chain.clone(), keypair.public_key).await;
        assert_eq!(1, resp_body.proofs.len());
        let proof = resp_body.proofs.first().unwrap();
        assert_eq!(proof.content, json!({"test2": "new kv"}));
    }

    #[tokio::test]
    async fn test_newly_upload_to_arweave_and_query() {
        let keypair = Secp256k1KeyPair::generate();
        let platform: String = Faker.fake();
        let identity: String = Faker.fake();
        let mut new_kv_chain = create_new_kv_chain(
            keypair.public_key, &platform, &identity, json!({"test": "123"}));
        new_kv_chain.signature = new_kv_chain.sign(&keypair).unwrap();

        let resp_body = create_req_and_send(new_kv_chain.clone(), keypair.public_key).await;
        assert_eq!(1, resp_body.proofs.len());

        // we check the arweave_id from the response whether is same with database.
        let from_response_arweave_id = resp_body.proofs.first().unwrap().arweave_id.clone();
        assert!(from_response_arweave_id != None);

        // take arweave id from kv_chains table
        let mut conn = establish_connection();
        let kv_chain_vec = find_all_by_identity(&mut conn, &platform, &identity).unwrap();
        assert_eq!(1, kv_chain_vec.len());
        assert_eq!(from_response_arweave_id, kv_chain_vec[0].arweave_id);
    }

    #[tokio::test]
    async fn test_multi_modify_after_arweave_id() {
        let keypair = Secp256k1KeyPair::generate();
        let mut conn = establish_connection();
        let platform: String = Faker.fake();
        let identity: String = Faker.fake();

        let mut first_kv_chain = create_new_kv_chain(
            keypair.public_key, &platform, &identity, json!({"first": "first"}));
        first_kv_chain.signature = first_kv_chain.sign(&keypair).unwrap();

        let resp_body = create_req_and_send(first_kv_chain.clone(), keypair.public_key).await;
        assert_eq!(1, resp_body.proofs.len());
        let primitive_arweave_id_in_kv = resp_body.proofs.first().unwrap().arweave_id.clone();
        
        let mut second_kv_chain = create_new_kv_chain(
            keypair.public_key, &platform, &identity, json!({"second": "second"}));
        let last_link = KVChain::find_last_link(&mut conn, &keypair.public_key).unwrap();
        second_kv_chain.previous_id = if let Some(last_link_instance) = last_link {
            Some(last_link_instance.id)
        } else {
            None
        };

        second_kv_chain.signature = second_kv_chain.sign(&keypair).unwrap();

        let resp_body = create_req_and_send(second_kv_chain.clone(), keypair.public_key).await;
        assert_eq!(1, resp_body.proofs.len());
        let current_arweave_id_in_kv = resp_body.proofs.first().unwrap().arweave_id.clone();
        assert_ne!(primitive_arweave_id_in_kv, current_arweave_id_in_kv);

        // take arweave id from kv_chains table
        let kv_chain_vec = find_all_by_identity(&mut conn, &platform, &identity).unwrap();
        assert_eq!(2, kv_chain_vec.len());
        assert_eq!(primitive_arweave_id_in_kv, kv_chain_vec[0].arweave_id);
        assert_eq!(current_arweave_id_in_kv, kv_chain_vec[1].arweave_id);
    }

    // NOTE: test below is created with `persona:` sig payload.
    // #[tokio::test]
    // async fn test_actual_case_1() {
    //     let req_body = UploadRequest{
    //         persona: Some("0x0289689d4846db795310b3fb6dea7ab8aba2b6734ddd3b3744a412ab174bf8cbfc".into()),
    //         avatar: None,
    //         platform: "twitter".into(),
    //         identity: "weipingzhu2".into(),
    //         signature: "De/UN6E7HosqZxhpG3+CRD7m8T+ozcdvKO/JCXTr/X9Hek0KP2SQFZQtZQOv/F9XgwufvHeGyD387I7QwJAxqRs=".into(),
    //         uuid: "fd042b27-0f21-476d-9e23-478c98ac6700".into(),
    //         created_at: 1650007736,
    //         patch: json!({
    //             "com.mask.plugin": {
    //                 "twitter_weipingzhu2": {
    //                     "nickname": "vitalik.eth",
    //                     "userId": "WeipingZhu2",
    //                     "imageUrl": "https://pbs.twimg.com/profile_images/1514868277415084038/BJSpRyjq_normal.png",
    //                     "avatarId": "1514868277415084038",
    //                     "address": "0x495f947276749ce646f68ac8c248420045cb7b5e",
    //                     "tokenId": "84457744602723809043049191225279009657327463478214710277063869711841964851201"
    //                 }
    //             }
    //         }),
    //     };
    //     let req: Request = ::http::Request::builder()
    //         .method(Method::POST)
    //         .uri(format!("http://localhost/test"))
    //         .body(serde_json::to_string(&req_body).unwrap())
    //         .unwrap();
    //     let resp = controller(req).await.unwrap();
    //     assert_eq!(resp.status(), StatusCode::CREATED);
    // }
}
