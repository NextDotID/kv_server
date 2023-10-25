#[cfg(test)]
mod tests {

    use std::{path::PathBuf, str::FromStr};

    use arweave_rs::{Arweave, crypto::base64::Base64, network::NetworkInfoClient};
    use url::Url;
    use uuid::Uuid;

    use crate::model::arweave::KVChainArweaveDocument;
    use crate::config::C;
    use crate::util::naive_now;


    fn prepare_test_document() -> KVChainArweaveDocument {
        KVChainArweaveDocument{
            avatar: "sample".into(),
            uuid: Uuid::new_v4(),
            persona: vec![],
            platform: "twitter".into(),
            identity: "".into(),
            patch: "".into(),
            signature: vec![],
            created_at: naive_now(),
            signature_payload: "".into(),
            previous_id: None,
            previous_arweave_id: None,
        }
    }

    /// We will test current network is work or not
    #[tokio::test]
    async fn test_network_work() {
        if C.arweave.is_none() {
            return;
        }
        let arweave_config = C.arweave.clone().unwrap();
        let arweave_url = Url::parse(&arweave_config.url).unwrap();
        let network_client = NetworkInfoClient::new(arweave_url);
        let _ = match network_client.network_info().await {
            Ok(message) => message,
            Err(e) => panic!("{}", e),
        };
    }

    /// Check the upload function and status of upload.
    #[tokio::test]
    async fn test_send_data_to_arweave_and_check() {
        if C.arweave.is_none() {
            return;
        }
        let arweave_config = C.arweave.clone().unwrap();
        let test_document = prepare_test_document();
        let transcation_id = test_document.upload_to_arweave().await.unwrap();

        let arweave_url = Url::parse(&arweave_config.url).unwrap();
        let arweave_connect = Arweave::from_keypair_path(
            PathBuf::from(&arweave_config.jwt),
            arweave_url.clone()
        ).unwrap();

        let result = arweave_connect.get_tx_status(Base64::from_str(transcation_id.as_str()).unwrap()).await;

        // check this transaction is uploaded successful or not
        let status_code = result.unwrap().0;
        assert!(status_code == 200 || status_code == 202);
    }

}
