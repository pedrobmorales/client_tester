use anyhow::Result;
use serde::{Deserialize, Serialize};
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRouteRequest {
    pub routes: Vec<Route>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Route {
    pub prefix: String,
    pub enabled: bool,
    pub advertised: bool,
    pub is_primary: bool,
}

pub fn create_exit_route_request() -> CreateRouteRequest {
    CreateRouteRequest {
        routes: vec![
            Route {
                prefix: "0.0.0.0/0".to_string(),
                enabled: true,
                advertised: true,
                is_primary: true,
            },
            Route {
                prefix: "::/0".to_string(),
                enabled: true,
                advertised: true,
                is_primary: true,
            },
        ],
    }
}

pub async fn make_internet_gateway(
    machine_id: &str,
    ninja_panda_api_url: &str,
    api_key: &str,
    client: &reqwest::Client,
) -> Result<()> {
    let mid = &machine_id[8..]; //remove "machine:" prefix from the machineID
    let url = format!("{ninja_panda_api_url}/api/v1/machine/{mid}/routes");
    let route_request = create_exit_route_request();
    let res = client
        .post(url)
        .bearer_auth(api_key)
        .json(&route_request)
        .send()
        .await?;
    dbg!(&res);
    Ok(())
}
