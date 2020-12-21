use reqwest::{header, Client as ReqwestClient, Error};
use serde_json::Value;
use std::borrow::ToOwned;

pub type PatreonError = Error;

#[derive(Clone)]
pub struct Client {
    client: ReqwestClient,
}

impl Client {
    pub fn new(patreon_key: &str) -> Self {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(patreon_key).unwrap(),
        );
        let client = ReqwestClient::builder()
            .default_headers(headers)
            .build()
            .unwrap();
        Self { client }
    }

    pub async fn get_patron(
        &self,
        discord_id: u64,
    ) -> Result<(Option<String>, Option<String>), PatreonError> {
        let mut link = Some("https://www.patreon.com/api/oauth2/v2/campaigns/3229705/members?include=currently_entitled_tiers,user&fields%5Buser%5D=social_connections".to_string());
        let mut patreon_id: Option<String> = None;
        while link.is_some() {
            let res: Value = self
                .client
                .get(&link.unwrap())
                .send()
                .await?
                .json::<Value>()
                .await?;
            let info = res["included"].as_array().unwrap();
            let users = res["data"].as_array().unwrap();
            for user in info {
                if user["type"].as_str().unwrap().eq("user") {
                    let disc = &user["attributes"]["social_connections"]["discord"]["user_id"];
                    if let Some(Ok(disc)) = disc.as_str().map::<Result<u64, _>, _>(str::parse) {
                        if disc == discord_id {
                            patreon_id = Some(user["id"].as_str().unwrap().to_string());
                            for u in users {
                                if u["relationships"]["user"]["data"]["id"].as_str().unwrap()
                                    == patreon_id.as_ref().unwrap()
                                {
                                    let tiers = u["relationships"]["currently_entitled_tiers"]
                                        ["data"]
                                        .as_array()
                                        .unwrap();
                                    if !tiers.is_empty() {
                                        return Ok((
                                            patreon_id,
                                            tiers[0]["id"].as_str().map(ToOwned::to_owned),
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }
            }
            link = res["links"]["next"].as_str().map(ToOwned::to_owned);
        }
        Ok((patreon_id, None))
    }
}
