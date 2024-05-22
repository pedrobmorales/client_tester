use anyhow::{bail, Context, Result};

use bollard::{
    container::{AttachContainerOptions, CreateContainerOptions, LogOutput, StartContainerOptions},
    exec::{CreateExecOptions, StartExecResults},
    network::ConnectNetworkOptions,
    secret::HostConfig,
    service::ContainerCreateResponse,
    Docker,
};

use futures::StreamExt;
use reqwest::Client;
use serde_json::from_str;
use tokio::time::{self, sleep};

use crate::{
    execute_callback, get_labels,
    models::{status::StatusResult, ztcon::ConResult, ztn::NetMap},
    ninjapanda::create_namespace,
    users::get_user,
    Config, ExecuteCallbackRequest, RuntimeInformation,
};

use crate::containers::remove_container;

const OLD_MODE: &str = "USE_OLD_MODE";
pub const NGINX_NP_URL: &str = "http://ztclient_nginx:80";

pub mod states {
    pub const RUNNING_STATE: &str = "Running";
    pub const NEEDS_LOGIN_STATE: &str = "NeedsLogin";
}

pub async fn start_ztclientd(
    docker: &Docker,
    config: &Config,
    container_name: &str,
) -> Result<ContainerCreateResponse> {
    remove_container(docker, container_name).await;
    let tun_arg = "--tun";
    let userspace_arg = "userspace-networking";
    let statedir_arg = "--statedir";
    let statedir_path = "/var/lib/ztclientd";

    let debug_map_val = true; // var("ZT_DEBUG_MAP").is_ok_and(|x| x.eq_ignore_ascii_case("true"));
    let debug_register_val = true; //var("ZT_DEBUG_REGISTER").is_ok_and(|x| x.eq_ignore_ascii_case("true"));
    let debug_register = format!("ZT_DEBUG_REGISTER={}", debug_register_val);
    let debug_map = format!("ZT_DEBUG_MAP={}", debug_map_val);
    let create_container_options = bollard::container::Config {
        image: Some(config.ztclient_image.as_str()),
        labels: Some(get_labels()),
        hostname: Some(container_name),
        env: Some(vec![
            debug_register.as_str(),
            debug_map.as_str(),
            "NETTY_PORT=80",
        ]),
        cmd: Some(vec![tun_arg, userspace_arg, statedir_arg, statedir_path]),
        host_config: Some(HostConfig {
            cap_add: Some(vec!["NET_ADMIN".to_string(), "NET_RAW".to_string()]),
            ..Default::default()
        }),
        ..Default::default()
    };
    // Create the container
    let container = docker
        .create_container(
            Some(CreateContainerOptions {
                name: container_name,
                ..Default::default()
            }),
            create_container_options,
        )
        .await?;

    let network_options = ConnectNetworkOptions {
        container: container_name,
        ..Default::default()
    };

    let _attach_results = docker
        .attach_container(
            container_name,
            Some(AttachContainerOptions::<String> {
                stderr: Some(true),
                stdout: Some(true),
                logs: Some(true),
                detach_keys: Some("ctrl-c".to_string()),
                ..Default::default()
            }),
        )
        .await?;

    docker
        .connect_network(&config.docker_network_name, network_options)
        .await?;
    // Start the container
    docker
        .start_container(&container.id, None::<StartContainerOptions<String>>)
        .await?;

    Ok(container)
}

fn get_connect_actions() -> (&'static str, &'static str) {
    let old_value = std::env::var(OLD_MODE).unwrap_or("".to_string());
    // ("login-server", "up");
    // for 0.5.0 it's url, connect
    match old_value.as_str() {
        "true" => ("login-server", "up"),
        _ => ("url", "connect"),
    }
}
pub async fn ztclient_registration(docker: &Docker, container_name: &str) -> Result<String> {
    let (url_arg_name, connect_arg_name) = get_connect_actions();
    let server_url = format!("--{url_arg_name}={NGINX_NP_URL}");

    let client_hostname = format!("--hostname={}", container_name);
    let res = docker
        .create_exec(
            container_name,
            CreateExecOptions {
                cmd: Some(vec![
                    "ztclient",
                    connect_arg_name,
                    server_url.as_str(),
                    client_hostname.as_str(),
                ]),
                attach_stderr: Some(true),
                attach_stdout: Some(true),
                ..Default::default()
            },
        )
        .await;
    let result = res.unwrap();
    let output_result = docker.start_exec(result.id.as_str(), None).await?;
    let mut vect = Vec::new();
    match output_result {
        StartExecResults::Attached { output, input: _ } => {
            let mut dereffed = output;
            let output_line = dereffed.next().await;
            match output_line {
                Some(Ok(LogOutput::StdErr { message })) => {
                    vect.push(message);
                    ""
                }
                _ => "",
            };
        }
        StartExecResults::Detached => (),
    };
    let auth_url = if let Some(auth_url) = vect.last() {
        let line = std::str::from_utf8(auth_url).unwrap();
        let index = line.find("state=").expect("state= must be there");
        &line[index + 6..line.len()]
    } else {
        ""
    };
    Ok(String::from(auth_url))
}

pub async fn ztclient_registration_nh(docker: &Docker, container_name: &str) -> Result<String> {
    let (url_arg_name, connect_arg_name) = get_connect_actions();
    let server_url = format!("--{url_arg_name}={NGINX_NP_URL}");

    let res = docker
        .create_exec(
            container_name,
            CreateExecOptions {
                cmd: Some(vec!["ztclient", connect_arg_name, server_url.as_str()]),
                attach_stderr: Some(true),
                attach_stdout: Some(true),
                ..Default::default()
            },
        )
        .await;
    let result = res.unwrap();
    let output_result = docker.start_exec(result.id.as_str(), None).await?;
    let mut vect = Vec::new();
    match output_result {
        StartExecResults::Attached { output, input: _ } => {
            let mut dereffed = output;
            let output_line = dereffed.next().await;
            match output_line {
                Some(Ok(LogOutput::StdErr { message })) => {
                    vect.push(message);
                    ""
                }
                _ => "",
            };
        }
        StartExecResults::Detached => (),
    };
    let auth_url = if let Some(auth_url) = vect.last() {
        let line = std::str::from_utf8(auth_url).unwrap();
        let index = line.find("state=").expect("state= must be there");
        &line[index + 6..line.len()]
    } else {
        ""
    };
    Ok(String::from(auth_url))
}

pub async fn ztclient_register_forcereauth(
    docker: &Docker,
    container_name: &str,
) -> Result<String> {
    let (url_arg_name, connect_arg_name) = get_connect_actions();
    let server_url = format!("--{url_arg_name}={NGINX_NP_URL}");

    let client_hostname = format!("--hostname={}", container_name);
    let res = docker
        .create_exec(
            container_name,
            CreateExecOptions {
                cmd: Some(vec![
                    "ztclient",
                    connect_arg_name,
                    server_url.as_str(),
                    client_hostname.as_str(),
                    "--force-reauth",
                ]),
                attach_stderr: Some(true),
                attach_stdout: Some(true),
                ..Default::default()
            },
        )
        .await;
    let result = res.unwrap();
    let output_result = docker.start_exec(result.id.as_str(), None).await?;
    let mut vect = Vec::new();
    match output_result {
        StartExecResults::Attached { output, input: _ } => {
            let mut dereffed = output;
            let output_line = dereffed.next().await;
            match output_line {
                Some(Ok(LogOutput::StdErr { message })) => {
                    vect.push(message);
                    ""
                }
                _ => "",
            };
        }
        StartExecResults::Detached => (),
    };
    let auth_url = if let Some(auth_url) = vect.last() {
        let line = std::str::from_utf8(auth_url).unwrap();
        let index = line.find("state=").expect("state= must be there");
        &line[index + 6..line.len()]
    } else {
        ""
    };
    Ok(String::from(auth_url))
}

pub async fn ztclient_alternate_hostname_registration(
    docker: &Docker,
    container_name: &str,
    hostname: &str,
) -> Result<String> {
    let (url_arg_name, connect_arg_name) = get_connect_actions();
    let server_url = format!("--{url_arg_name}={NGINX_NP_URL}");

    let client_hostname = format!("--hostname={}", hostname);
    let res = docker
        .create_exec(
            container_name,
            CreateExecOptions {
                cmd: Some(vec![
                    "ztclient",
                    connect_arg_name,
                    server_url.as_str(),
                    client_hostname.as_str(),
                ]),
                attach_stderr: Some(true),
                attach_stdout: Some(true),
                ..Default::default()
            },
        )
        .await;
    let result = res.unwrap();
    let output_result = docker.start_exec(result.id.as_str(), None).await?;
    let mut vect = Vec::new();
    match output_result {
        StartExecResults::Attached { output, input: _ } => {
            let mut dereffed = output;
            let output_line = dereffed.next().await;
            match output_line {
                Some(Ok(LogOutput::StdErr { message })) => {
                    vect.push(message);
                    ""
                }
                _ => "",
            };
        }
        StartExecResults::Detached => (),
    };
    let auth_url = if let Some(auth_url) = vect.last() {
        let line = std::str::from_utf8(auth_url).unwrap();
        let index = line.find("state=").expect("state= must be there");
        &line[index + 6..line.len()]
    } else {
        ""
    };
    Ok(String::from(auth_url))
}

pub async fn ztclient_status_json(docker: &Docker, container_name: &str) -> Result<StatusResult> {
    let res = docker
        .create_exec(
            container_name,
            CreateExecOptions {
                cmd: Some(vec!["ztclient", "status", "--json"]),
                attach_stderr: Some(true),
                attach_stdout: Some(true),
                ..Default::default()
            },
        )
        .await;
    let result = res.unwrap();
    let output_result = docker.start_exec(result.id.as_str(), None).await?;
    let mut vect = Vec::new();
    match output_result {
        StartExecResults::Attached { output, input: _ } => {
            let mut dereffed = output;
            let output_line = dereffed.next().await;
            match output_line {
                Some(Ok(LogOutput::StdOut { message })) => {
                    vect.push(message);
                    ""
                }

                _ => "",
            };
        }
        StartExecResults::Detached => (),
    };

    let json = if let Some(json) = vect.last() {
        let line = std::str::from_utf8(json).unwrap();
        let as_json: StatusResult = serde_json::from_str(line)
            .with_context(|| "Unable to unmarshall")
            .unwrap();
        as_json
    } else {
        bail!("Not a ztclient status json root")
    };
    Ok(json)
}

pub async fn preauth_token_registration(
    docker: &Docker,
    container_name: &str,
    preauth_token: &str,
    np_url: &str,
) -> Result<String> {
    let (url_arg_name, connect_arg_name) = get_connect_actions();
    let server_url = format!("--{url_arg_name}={np_url}");

    let client_hostname = format!("--hostname={}", container_name);

    let commands = vec![
        "ztclient",
        connect_arg_name,
        server_url.as_str(),
        "--auth-token",
        preauth_token,
        client_hostname.as_str(),
    ];
    dbg!(&commands);

    let res = docker
        .create_exec(
            container_name,
            CreateExecOptions {
                cmd: Some(commands),
                attach_stderr: Some(true),
                attach_stdout: Some(true),
                ..Default::default()
            },
        )
        .await;
    let result = res.unwrap();
    let output_result = docker.start_exec(result.id.as_str(), None).await?;
    let mut vect = Vec::new();
    match output_result {
        StartExecResults::Attached { output, input: _ } => {
            let mut dereffed = output;
            let output_line = dereffed.next().await;
            match output_line {
                Some(Ok(LogOutput::StdErr { message })) => {
                    vect.push(message);
                    ""
                }
                _ => "",
            };
        }
        StartExecResults::Detached => (),
    };
    let output = if let Some(auth_url) = vect.last() {
        std::str::from_utf8(auth_url).unwrap()
    } else {
        ""
    };
    Ok(String::from(output))
}

pub async fn ztclient_status(docker: &Docker, container_name: &str) -> Result<String> {
    let res = docker
        .create_exec(
            container_name,
            CreateExecOptions {
                cmd: Some(vec!["ztclient", "status"]),
                attach_stderr: Some(true),
                attach_stdout: Some(true),
                ..Default::default()
            },
        )
        .await;
    let result = res.unwrap();
    let output_result = docker.start_exec(result.id.as_str(), None).await?;
    let mut vect = Vec::new();
    match output_result {
        StartExecResults::Attached { output, input: _ } => {
            let mut dereffed = output;
            let output_line = dereffed.next().await;
            match output_line {
                Some(Ok(LogOutput::StdErr { message })) => {
                    vect.push(message);
                    ""
                }
                Some(Ok(LogOutput::StdOut { message })) => {
                    vect.push(message);
                    ""
                }
                _ => "",
            };
        }
        StartExecResults::Detached => (),
    };
    let full_string: String = vect
        .iter()
        .map(|x| std::str::from_utf8(x).unwrap())
        .map(str::to_string)
        .collect::<Vec<String>>()
        .join("\n");
    Ok(full_string)
}

pub async fn ztclient_logout(docker: &Docker, container_name: &str) -> Result<Vec<String>> {
    let res = docker
        .create_exec(
            container_name,
            CreateExecOptions {
                cmd: Some(vec!["ztclient", "logout"]),
                attach_stderr: Some(true),
                attach_stdout: Some(true),
                ..Default::default()
            },
        )
        .await;
    let result = res.unwrap();
    let output_result = docker.start_exec(result.id.as_str(), None).await?;
    let mut vect = Vec::new();
    match output_result {
        StartExecResults::Attached { output, input: _ } => {
            let mut dereffed = output;
            let output_line = dereffed.next().await;
            match output_line {
                Some(Ok(LogOutput::StdErr { message })) => {
                    vect.push(message);
                    ""
                }
                Some(Ok(LogOutput::StdOut { message })) => {
                    vect.push(message);
                    ""
                }
                _ => "",
            };
        }
        StartExecResults::Detached => (),
    };
    let full_string: Vec<String> = vect
        .iter()
        .map(|x| std::str::from_utf8(x).unwrap())
        .map(str::to_string)
        .collect();
    Ok(full_string)
}

pub async fn ztclient_execute(
    docker: &Docker,
    container_name: &str,
    commands: Vec<&str>,
) -> Result<Vec<String>> {
    let res = docker
        .create_exec(
            container_name,
            CreateExecOptions {
                cmd: Some(commands),
                attach_stderr: Some(true),
                attach_stdout: Some(true),
                ..Default::default()
            },
        )
        .await;
    let result = res.unwrap();
    let output_result = docker.start_exec(result.id.as_str(), None).await?;
    let mut vect = Vec::new();
    match output_result {
        StartExecResults::Attached { output, input: _ } => {
            let mut dereffed = output;
            let output_line = dereffed.next().await;
            match output_line {
                Some(Ok(LogOutput::StdErr { message })) => {
                    vect.push(message);
                    ""
                }
                Some(Ok(LogOutput::StdOut { message })) => {
                    vect.push(message);
                    ""
                }
                Some(Ok(LogOutput::Console { message })) => {
                    vect.push(message);
                    ""
                }
                Some(Ok(LogOutput::StdIn { message })) => {
                    dbg!("Logoutput stdin");
                    vect.push(message);
                    ""
                }
                Some(Err(x)) => {
                    dbg!(&x);
                    ""
                }
                None => {
                    dbg!("None case")
                }
            };
        }
        StartExecResults::Detached => (),
    };
    let full_string: Vec<String> = vect
        .iter()
        .map(|x| std::str::from_utf8(x).unwrap())
        .map(str::to_string)
        .collect();
    Ok(full_string)
}

pub async fn create_and_register_client(
    runtime_info: &RuntimeInformation,
    docker: &Docker,
    config: &Config,
    client: &Client,
    container_name: &str,
    namespace_name: &str,
    user_info_id: usize,
) -> Result<String> {
    start_ztclientd(docker, config, container_name)
        .await
        .unwrap();
    let correlation_id = ztclient_registration(docker, container_name).await.unwrap();
    let request = ExecuteCallbackRequest {
        correlation_id: correlation_id.as_str(),
        api_key: &runtime_info.ninja_panda_api_key,
        namespace_name,
        user_info_id,
        ninja_panda_api_url: &runtime_info.ninja_panda_api_url,
    };
    let machine_id = execute_callback(client, &request).await.unwrap();
    Ok(machine_id)
}

pub async fn create_client_with_preauth_token(
    docker: &Docker,
    config: &Config,
    container_name: &str,
    preauth_token: &str,
) -> Result<String> {
    start_ztclientd(docker, config, container_name)
        .await
        .unwrap();
    let preauth_result =
        preauth_token_registration(docker, container_name, preauth_token, NGINX_NP_URL)
            .await
            .unwrap();
    Ok(preauth_result)
}

pub async fn wait_for_peer(
    docker: &Docker,
    container_name: &str,
    peer_name: &str,
) -> Result<ConResult> {
    let exec = ztclient_execute(
        docker,
        container_name,
        vec!["ztclient", "examine", "wait-for-peer", "--peer", peer_name],
    )
    .await
    .unwrap();
    let str1 = exec.into_iter().next().unwrap();
    let con_res: ConResult = from_str(str1.as_str())
        .with_context(|| "Unable to unmarshal a ZTCon Result")
        .unwrap();
    Ok(con_res)
}

pub async fn wait_for_state_change(
    docker: &Docker,
    container_name: &str,
    new_state: &str,
) -> StatusResult {
    let mut counter = 0;
    let status: StatusResult = loop {
        let status = ztclient_status_json(docker, container_name).await.unwrap();
        if status.backend_state == new_state {
            break status;
        }
        counter += 1;
        if counter >= 1000 {
            panic!("Counter taking too long!");
        }
        let sleep_time = time::Duration::from_millis(500);
        sleep(sleep_time).await;
    };
    status
}

pub async fn ztclient_netmap(docker: &Docker, container_name: &str) -> NetMap {
    let command = vec!["ztclient", "examine", "netmap"];

    let strings = ztclient_execute(docker, container_name, command)
        .await
        .unwrap();
    let netmap_str = strings.first().unwrap();
    let ztn_netmap: NetMap = from_str(netmap_str).expect("Unable to unmarshall NetMap");
    ztn_netmap
}

pub async fn create_running_clients(
    runtime_info: &RuntimeInformation,
    docker: &Docker,
    config: &Config,
    client: &Client,
    hostname_prefix: String,
    num_clients: usize,
    namespace_name: &str,
) -> Vec<String> {
    create_namespace(namespace_name, runtime_info, client)
        .await
        .unwrap();

    for x in 1..num_clients + 1 {
        let name = format!("{}{:0>3}", hostname_prefix, x);
        start_ztclientd(docker, config, name.as_str())
            .await
            .unwrap();
        let correlation_id = ztclient_registration(docker, name.as_str()).await.unwrap();
        let request = ExecuteCallbackRequest {
            correlation_id: correlation_id.as_str(),
            api_key: &runtime_info.ninja_panda_api_key,
            namespace_name,
            user_info_id: (x % 9),
            ninja_panda_api_url: &runtime_info.ninja_panda_api_url,
        };
        execute_callback(client, &request).await.unwrap();
    }

    // Check that the userInfo is correct for newly created nodes that have no peers.
    for x in 1..num_clients + 1 {
        let name = format!("{}{:0>3}", hostname_prefix, x);
        let user_info = get_user(x % 9);

        let status = wait_for_state_change(docker, name.as_str(), "Running").await;
        let assigned_user_id = status.self_field.user_id;

        let user_map = status.user.unwrap();
        let user_object = user_map.get(&assigned_user_id.to_string()).unwrap();
        user_object.assert_eq(&user_info);
    }

    let mut container_names = Vec::new();
    for x in 1..num_clients + 1 {
        let name = format!("{}{:0>3}", hostname_prefix, x);
        container_names.push(name);
    }
    container_names
}
