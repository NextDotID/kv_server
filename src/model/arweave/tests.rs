#[cfg(test)]
mod tests {

    use std::{path::PathBuf, str::FromStr}; 

    use arweave_rs::{Arweave, crypto::base64::Base64, network::NetworkInfoClient};
    use chrono::NaiveDate;
    use url::Url;
    use uuid::Uuid;

    use crate::model::arweave::KVChainArweaveDocument;
    use crate::config::C;


    fn prepare_test_document() -> KVChainArweaveDocument {
        KVChainArweaveDocument{
            avatar: "sample".into(),
            uuid: Uuid::new_v4(),
            persona: vec![],
            platform: "twitter".into(),
            identity: "".into(),
            patch: "".into(),
            signature: vec![],
            created_at: NaiveDate::from_ymd_opt(2023, 8, 8).unwrap().and_hms_milli_opt(12, 34, 56, 7).unwrap(),
            signature_payload: "".into(),
            previous_id: None,
            previous_arweave_id: None,
        }
    }

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
        let test_document = prepare_test_document();
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
    // #[tokio::test]
    // async fn test_check_data() {
    //     use std::thread::sleep;
    //     use std::time::Duration;
    //     use http::Uri;
    //     use hyper::Client;
    //     use hyper_tls::HttpsConnector;

    //     let upload_document = prepare_test_document();
    //     let transcation_id = upload_document.clone().upload_to_arweave().await.unwrap();
    //     // let transcation_id = "lB9_qabnusXeunR57Qk8oP9_-pPkEkoajNGTOlj082Y".to_string();
    //     let url = format!("https://arweave.net/tx/{}/data", transcation_id).parse::<Uri>().unwrap();

    //     let https = HttpsConnector::new();
    //     let client = Client::builder().build::<_, hyper::Body>(https);
    //     loop {
    //         match client.get(url.clone()).await {
    //             Ok(mut resp) => {
    //                 if resp.status() == 200 {
    //                     let body = hyper::body::to_bytes(resp.body_mut()).await.unwrap();
    //                     let encode_str = std::str::from_utf8(&body).unwrap().to_string();
    //                     let decoded = base64::decode(encode_str).unwrap();
    //                     assert_eq!(decoded, serde_json::to_vec(&upload_document).unwrap());
    //                     break;
    //                 } else if resp.status().is_server_error() {
    //                     println!("Server error!");
    //                 } else {
    //                     println!("Received response: {:?}", resp.status());
    //                 }
    //             }
    //             Err(e) => println!("Request failed: {:?}", e),
    //         }
    //         sleep(Duration::from_secs(2));
    //     }
    // }


}