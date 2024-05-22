use anyhow::{bail, Result};
use bollard::Docker;
use containers::remove_container;
use ninjapanda::delete_machine;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs::read_to_string};
use ztclient::{start_ztclientd, ztclient_registration, ztclient_registration_nh};

use crate::models::ExecuteCallbackResponse;

pub mod containers;
pub mod errors;
pub mod intgates;
pub mod models;
pub mod ninjapanda;
pub mod users;
pub mod ztclient;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub ninja_panda_container_name: String,
    pub postgres_container_name: String,
    pub postgres_user: String,
    pub postgres_name: String,
    pub postgres_db: String,
    pub postgres_password: String,
    pub docker_network_name: String,
    pub ztclient_image: String,
    pub ninja_postgres_name: String,
    pub ninja_postgres_password: String,
    pub ninja_machine_auth_url: String,
    pub kafka_container_name: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeInformation {
    /// API Key used to interact with Ninja Panda
    pub ninja_panda_api_key: String,
    /// The URL to Ninja Panda's API
    pub ninja_panda_api_url: String,
}

/// Read the running.json file at the root of the runtime tree after an environment has
/// been started, and returns a Result with the runtime information structure that was
/// parsed from the file.
pub fn get_running_json() -> Result<RuntimeInformation> {
    // In the case of our tests the path is two levels deep.
    let paths = ["./running.json", "../../running.json"];
    for path in paths.iter() {
        let check_file = read_to_string(path);
        if let Ok(file_text) = check_file {
            let runtime_information: RuntimeInformation =
                serde_json::from_str(file_text.as_str()).unwrap();
            return Ok(runtime_information);
        }
    }
    bail!("Unable to read runtime information");
}

pub fn get_unique_timestamp() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos()
}
pub struct ExecuteCallbackRequest<'a> {
    pub correlation_id: &'a str,
    pub api_key: &'a str,
    pub namespace_name: &'a str,
    pub ninja_panda_api_url: &'a str,
    pub user_info_id: usize,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterCallbackRequest {
    pub namespace: String,
    pub user_info: users::UserInfo,
}

pub async fn execute_callback<'a>(
    client: &reqwest::Client,
    request: &ExecuteCallbackRequest<'a>,
) -> Result<String> {
    let user_info = users::get_user(request.user_info_id);
    let reg_request = RegisterCallbackRequest {
        namespace: request.namespace_name.to_owned(),
        user_info,
    };
    let _json_request = serde_json::to_string(&reg_request).unwrap();

    let url = format!(
        "{}/api/v1/machine/register/callback/{}",
        request.ninja_panda_api_url, request.correlation_id
    );
    let _res = client
        .post(url)
        .bearer_auth(request.api_key)
        .json(&reg_request)
        .send()
        .await?;
    assert_eq!(
        200,
        _res.status(),
        "Register callback POST call did not return 200"
    );
    let _response: ExecuteCallbackResponse = _res.json().await.unwrap();
    let machine_id = _response.machine.machine_id;

    // TODO: When we add logging just log this body to the debug level.
    // let body: serde_json::Value = res.json().await?;
    Ok(machine_id)
}

/// These labels are added to the docker container.  These are what allow Docker Desktop to believe
/// that this is a Compose project, and store all the containers in the hierarchical structure that
/// comes when you use a compose.yaml file
pub fn get_labels() -> HashMap<&'static str, &'static str> {
    let mut labels = HashMap::new();
    labels.insert("com.docker.compose.project", "ztclient-test-tool");
    labels
}

pub fn random_container_name() -> String {
    use rand::{distributions::Alphanumeric, Rng}; // 0.8
    let s: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(7)
        .map(char::from)
        .collect();
    s.to_ascii_lowercase()
}

pub fn random_names(n: usize) -> Vec<String> {
    let mut v = Vec::new();
    for _ in 0..n {
        v.push(random_container_name());
    }
    v
}

pub async fn start_and_register_n_clients(
    docker: &Docker,
    runtime_info: &RuntimeInformation,
    config: &Config,
    client: &Client,
    namespace_name: &str,
    user_id: usize,
    container_names: &Vec<String>,
) {
    for container_name in container_names {
        start_and_register_client(
            docker,
            runtime_info,
            config,
            client,
            container_name,
            namespace_name,
            user_id,
        )
        .await
    }
}
pub async fn start_and_register_client(
    docker: &Docker,
    runtime_info: &RuntimeInformation,
    config: &Config,
    client: &Client,
    container_name: &str,
    namespace_name: &str,
    user_id: usize,
) {
    start_ztclientd(docker, config, container_name)
        .await
        .unwrap();
    let correlation_id = ztclient_registration(docker, container_name).await.unwrap();
    let request = ExecuteCallbackRequest {
        correlation_id: correlation_id.as_str(),
        api_key: &runtime_info.ninja_panda_api_key,
        namespace_name,
        user_info_id: user_id,
        ninja_panda_api_url: &runtime_info.ninja_panda_api_url,
    };
    execute_callback(client, &request).await.unwrap();
}

pub async fn start_and_register_client_nh(
    docker: &Docker,
    runtime_info: &RuntimeInformation,
    config: &Config,
    client: &Client,
    container_name: &str,
    namespace_name: &str,
    user_id: usize,
) {
    start_ztclientd(docker, config, container_name)
        .await
        .unwrap();
    let correlation_id = ztclient_registration_nh(docker, container_name)
        .await
        .unwrap();
    let request = ExecuteCallbackRequest {
        correlation_id: correlation_id.as_str(),
        api_key: &runtime_info.ninja_panda_api_key,
        namespace_name,
        user_info_id: user_id,
        ninja_panda_api_url: &runtime_info.ninja_panda_api_url,
    };
    execute_callback(client, &request).await.unwrap();
}

pub async fn container_cleanup(
    docker: &Docker,
    container_names: Vec<String>,
    machine_ids: Vec<String>,
    runtime_info: &RuntimeInformation,
    client: &reqwest::Client,
) {
    //TODO: I'd really like to have an env var or something to ke
    if true {
        return;
    }

    for x in container_names.iter() {
        remove_container(docker, x).await;
    }
    for machine_id in machine_ids {
        delete_machine(&machine_id, runtime_info, client)
            .await
            .unwrap();
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {
        let result = 4;
        assert_eq!(result, 4);
    }
}
