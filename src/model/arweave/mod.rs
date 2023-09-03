mod tests;

use std::path::PathBuf;

use ::uuid::Uuid;
use chrono::NaiveDateTime;
use arweave_rs::{Arweave, crypto::base64::Base64};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KVChainArweaveDocument {
    pub avatar: String,
    pub uuid: Uuid,
    pub persona: Vec<u8>,
    pub platform: String,
    pub identity: String,
    pub patch: serde_json::Value,
    pub signature: Vec<u8>,
    pub created_at: NaiveDateTime,
    pub signature_payload: String,
    pub previous_uuid: Option<Uuid>,
}    

impl KVChainArweaveDocument {

    pub async fn upload_to_arweave(self) -> Option<String> {
        // create the signer
        let arweave_url = Url::parse("https://arweave.net").unwrap();
        let arweave_connect = Arweave::from_keypair_path(
            PathBuf::from("/workspaces/kv_server/test.json"),
            arweave_url.clone()
        ).unwrap();

        // first get the previous_uuid and previous_arweave_id by the arweave_id
        let _wallet = arweave_rs::wallet::WalletInfoClient::new(arweave_url);
        let _address = arweave_connect.get_wallet_address().unwrap();
        
        let target = Base64(vec![]);
        let data = serde_json::to_vec(&self).unwrap();
        let fee = arweave_connect.get_fee(target.clone(), data.clone()).await.unwrap();
        let send_transaction = arweave_connect.create_transaction(
            target,
            vec![],
            data,
            0,
            fee,
            true
        ).await.unwrap();
        
        let signed_transaction = arweave_connect.sign_transaction(send_transaction).unwrap();
        let result = arweave_connect.post_transaction(&signed_transaction).await.unwrap();
        
        // return the transcation id to user
        Some(result.0)
    }
}

