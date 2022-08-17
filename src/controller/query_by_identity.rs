use crate::{
    controller::{query_parse, Request, Response},
    error::Error,
    model::{establish_connection, kv::find_all_by_identity},
};
use diesel::PgConnection;
use http::StatusCode;
use serde::{Deserialize, Serialize};

use super::json_response;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct QueryResponse {
    pub values: Vec<QueryResponseSingleAvatar>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct QueryResponseSingleAvatar {
    pub avatar: String,
    pub content: serde_json::Value,
}

pub async fn controller(req: Request) -> Result<Response, Error> {
    let params = query_parse(req);
    let platform = params
        .get("platform")
        .ok_or(Error::ParamMissing("platform".into()))?;
    let identity = params
        .get("identity")
        .ok_or(Error::ParamMissing("identity".into()))?;

    let conn = establish_connection();
    let response = query_response(&conn, &platform, &identity)?;

    json_response(StatusCode::OK, &response)
}

fn query_response(
    conn: &PgConnection,
    platform: &str,
    identity: &str,
) -> Result<QueryResponse, Error> {
    let found = find_all_by_identity(conn, platform, identity)?;
    let values: Vec<QueryResponseSingleAvatar> = found
        .into_iter()
        .map(|kv| QueryResponseSingleAvatar {
            avatar: kv.avatar(),
            content: kv.content,
        })
        .collect();

    Ok(QueryResponse { values })
}

#[cfg(test)]
mod tests {
    use fake::{Faker, Fake};
    use http::Method;
    use crate::{model::kv::find_or_create, crypto::secp256k1::Secp256k1KeyPair};
    use super::*;

    #[tokio::test]
    async fn test_smoke() -> Result<(), Error> {
        let platform = "twitter".to_string();
        let identity: String = Faker.fake();

        let req: Request = ::http::Request::builder()
            .method(Method::GET)
            .uri(format!("http://localhost/test?platform={}&identity={}", platform, identity))
            .body("".into())
            .unwrap();

        let resp = controller(req).await.unwrap();
        let body: QueryResponse= serde_json::from_str(resp.body()).unwrap();
        assert_eq!(0, body.values.len());

        Ok(())
    }

    #[tokio::test]
    async fn test_find_result() -> Result<(), Error> {
        let conn = establish_connection();
        let platform: String = "twitter".into();
        let identity: String = Faker.fake();
        let Secp256k1KeyPair {
            public_key: public_key_1,
            secret_key: _,
        } = Secp256k1KeyPair::generate();
        let Secp256k1KeyPair {
            public_key: public_key_2,
            secret_key: _,
        } = Secp256k1KeyPair::generate();
        let (created1, _) = find_or_create(&conn, &platform, &identity, &public_key_1).unwrap();
        let (created2, _) = find_or_create(&conn, &platform, &identity, &public_key_2).unwrap();
        let req: Request = ::http::Request::builder()
            .method(Method::GET)
            .uri(format!("http://localhost/test?platform={}&identity={}", platform, identity))
            .body("".into())
            .unwrap();
        let resp = controller(req).await.unwrap();
        let body: QueryResponse= serde_json::from_str(resp.body()).unwrap();
        assert_eq!(2, body.values.len());
        let avatars: Vec<String> = body.values.into_iter().map(|kv| kv.avatar).collect();
        assert!(avatars.contains(&created1.avatar()));
        assert!(avatars.contains(&created2.avatar()));

        Ok(())
    }
}
