#[cfg(test)]
mod tests {

    use std::{path::PathBuf, str::FromStr};

    use arweave_rs::{Arweave, crypto::base64::Base64, network::NetworkInfoClient};
    use chrono::{NaiveDate, NaiveTime, NaiveDateTime};
    use url::Url;
    use uuid::Uuid;

    use crate::model::arweave::KVChainArweaveDocument;
    use crate::config::C;

    /// We will test current network is work or not
    #[tokio::test]
    async fn test_network_work() {
        let arweave_url = Url::parse(C.arweave.url.as_str()).unwrap();
        let network_client = NetworkInfoClient::new(arweave_url);
        let _ = match network_client.network_info().await {
            Ok(message) => message,
            Err(e) => panic!("{}", e),
        };
    }

    /// Check the upload function and status of upload.
    #[tokio::test]
    async fn test_send_data_to_arweave_and_check() {
        let uuid = Uuid::new_v4();
        let test_document = KVChainArweaveDocument{
            avatar: "sample".into(),
            uuid: uuid,
            persona: vec![],
            platform: "twitter".into(),
            identity: "".into(),
            patch: "".into(),
            signature: vec![],
            created_at: NaiveDateTime::new(NaiveDate::from_ymd_opt(2023, 8, 8).unwrap(), NaiveTime::from_hms_milli_opt(12, 34, 56, 7).unwrap()),
            signature_payload: "".into(),
            previous_uuid: None,
        };
        let transcation_id = test_document.upload_to_arweave().await.unwrap();
        
        let arweave_url = Url::parse(C.arweave.url.as_str()).unwrap();
        let arweave_connect = Arweave::from_keypair_path(
            PathBuf::from(C.arweave.jwt.as_str()),
            arweave_url.clone()
        ).unwrap();

        let result = arweave_connect.get_tx_status(Base64::from_str(transcation_id.as_str()).unwrap()).await;

        // check this transaction is uploaded successful or not
        let status_code = result.unwrap().0;
        assert!(status_code == 200 || status_code == 202);
    }

    /// Check the upload data on arweave is correct or not
    #[tokio::test]
    async fn test_check_data() {

    }

}