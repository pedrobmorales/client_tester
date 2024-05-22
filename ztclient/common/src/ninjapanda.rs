use std::time::SystemTime;

use anyhow::Result;

use bollard::{
    container::LogOutput,
    exec::{CreateExecOptions, StartExecResults},
    Docker,
};

use futures::StreamExt;

use crate::{
    models::{
        Acl, AclPolicy, CreateAclPolicyRequest, CreateNamespaceRequest, CreatePreauthTokenRequest,
        GetMachinesResponse, Group, Machine, PreauthTokenResponse, UpdateAclPolicyRequest,
    },
    random_container_name, RuntimeInformation,
};

const PREAUTH_TOKEN_API: &str = "/api/v1/preauthkey";

pub async fn start_ninjapanda(docker: &Docker, container_name: &str) -> Result<String> {
    let apikeys_create = docker
        .create_exec(
            container_name,
            CreateExecOptions {
                attach_stdout: Some(true),
                cmd: Some(vec!["ninjapanda", "apikeys", "create"]),
                ..Default::default()
            },
        )
        .await?;
    let output_result = docker.start_exec(apikeys_create.id.as_str(), None).await?;
    let mut byteys = Vec::new();
    let api_key = match output_result {
        StartExecResults::Attached { output, input: _ } => {
            let mut dereffed = output;
            while let Some(f) = dereffed.next().await {
                if let Ok(LogOutput::StdOut { message }) = f {
                    byteys.push(message);
                }
            }
            let iter = byteys.iter().last();
            std::str::from_utf8(iter.unwrap())
                .unwrap()
                .replace('\n', "")
        }
        StartExecResults::Detached => String::new(),
    };
    Ok(api_key)
}

/// Queries Ninja Panda to get all the machines that have a hostname that starts with one of the desired hostnames and return an array of machine IDs.
pub async fn get_all_machine_ids(
    runtime_info: &RuntimeInformation,
    client: &reqwest::Client,
    desired_hostnames: &[String],
) -> Result<Vec<String>> {
    let res = client
        .get(format!(
            "{}/api/v1/machine",
            runtime_info.ninja_panda_api_url
        ))
        .bearer_auth(&runtime_info.ninja_panda_api_key)
        .send()
        .await?;

    let v: GetMachinesResponse = res.json().await?;
    let v_str = serde_json::to_string_pretty(&v).unwrap();
    println!("{}", v_str);
    let machine_ids = v
        .machines
        .into_iter()
        .filter(|m| {
            let hostname = &m.hostname;
            let count = desired_hostnames
                .iter()
                .filter(|x| hostname.starts_with(*x))
                .count();
            count > 0
        })
        .map(|m| format!("machine:{}", m.machine_id))
        .collect();
    dbg!("After filtering machine_ids", &machine_ids);
    Ok(machine_ids)
}

/// Queries Ninja Panda to get all the machines that have a hostname that starts with one of the desired hostnames and return an array of machine IDs.
pub async fn get_all_machines(
    runtime_info: &RuntimeInformation,
    client: &reqwest::Client,
    desired_hostnames: Vec<String>,
) -> Result<Vec<Machine>> {
    let res = client
        .get(format!(
            "{}/api/v1/machine",
            runtime_info.ninja_panda_api_url
        ))
        .bearer_auth(&runtime_info.ninja_panda_api_key)
        .send()
        .await?;

    let v: GetMachinesResponse = res.json().await?;
    let final_machines = v
        .machines
        .into_iter()
        .filter(|m| {
            let hostname = &m.hostname;
            let count = desired_hostnames
                .iter()
                .filter(|x| hostname.starts_with(*x))
                .count();
            count > 0
        })
        .collect();
    dbg!("After filtering machine_ids", &final_machines);
    Ok(final_machines)
}
/// Create an ACL Policy that allows all machines to see each other
pub async fn make_all_machines_peers(
    runtime_info: &RuntimeInformation,
    machine_ids: &[String],
    client: &reqwest::Client,
) -> Result<String> {
    use uuid::Uuid;
    let id = Uuid::new_v4();
    let id_g1 = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let id_g2 = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let g1_name = format!("group:{}", id_g1);
    let g2_name = format!("group:{}", id_g2);
    let g1 = Group {
        key: g1_name.to_string(),
        values: machine_ids.to_vec(),
    };
    let g2 = Group {
        key: g2_name.to_string(),
        values: machine_ids.to_vec(),
    };
    let acl1 = Acl {
        order: 0,
        action: "accept".to_string(),
        port: "*".to_string(),
        protocol: "tcp".to_string(),
        sources: vec![g2_name.to_string()],
        destinations: vec![g1_name.to_string()],
    };
    let acl2 = Acl {
        order: 1,
        action: "accept".to_string(),
        port: "*".to_string(),
        protocol: "icmp".to_string(),
        sources: vec![g2_name.to_string()],
        destinations: vec![g1_name.to_string()],
    };
    let create_acl_policy = CreateAclPolicyRequest {
        acl_policy: AclPolicy {
            aclpolicy_id: id.to_string(),
            order: "0".to_string(),
            groups: vec![g1, g2],
            acls: vec![acl1, acl2],
        },
    };
    let cpr_json = serde_json::to_string_pretty(&create_acl_policy).unwrap();
    println!("{}", cpr_json);

    let res = client
        .post(format!(
            "{}/api/v1/aclpolicy",
            runtime_info.ninja_panda_api_url
        ))
        .bearer_auth(&runtime_info.ninja_panda_api_key)
        .json(&create_acl_policy)
        .send()
        .await?;

    let status = res.status();

    assert!(
        status.is_success(),
        "Unable to create ACL Policies between all hosts"
    );

    res.text().await?;
    Ok(id.to_string())
}

pub async fn make_all_machines_png(
    runtime_info: &RuntimeInformation,
    machine_ids: &[String],
    client: &reqwest::Client,
) -> Result<String> {
    use uuid::Uuid;
    let id = Uuid::new_v4();
    let acl1 = Acl {
        order: 0,
        action: "accept".to_string(),
        port: "*".to_string(),
        protocol: "tcp".to_string(),
        sources: machine_ids.to_vec(),
        destinations: machine_ids.to_vec(),
    };
    let acl2 = Acl {
        order: 2,
        action: "accept".to_string(),
        port: "*".to_string(),
        protocol: "tcp".to_string(),
        sources: machine_ids.to_vec(),
        destinations: machine_ids.to_vec(),
    };
    let acl3: Acl = Acl {
        order: 1,
        action: "accept".to_string(),
        port: "*".to_string(),
        protocol: "tcp".to_string(),
        sources: machine_ids.to_vec(),
        destinations: machine_ids.to_vec(),
    };
    let create_acl_policy = CreateAclPolicyRequest {
        acl_policy: AclPolicy {
            aclpolicy_id: id.to_string(),
            order: "0".to_string(),
            groups: vec![],
            acls: vec![acl1, acl2, acl3],
        },
    };
    let _ignore = submit_policy_request(runtime_info, &create_acl_policy, &client).await;
    Ok(id.to_string())
}

pub async fn grant_one_directional_policy(
    runtime_info: &RuntimeInformation,
    machine_id1: &str,
    machine_id2: &str,
    port_number: u32,
    client: &reqwest::Client,
) -> Result<String> {
    use uuid::Uuid;
    let id = Uuid::new_v4();
    let id_g1 = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let id_g2 = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let g1_name = format!("group:{}", id_g1);
    let g2_name = format!("group:{}", id_g2);
    let g1 = Group {
        key: g1_name.to_string(),
        values: vec![format!("machine:{machine_id1}")],
    };
    let g2 = Group {
        key: g2_name.to_string(),
        values: vec![format!("machine:{machine_id2}")],
    };
    let acl1 = Acl {
        order: 0,
        action: "accept".to_string(),
        port: port_number.to_string(),
        protocol: "tcp".to_string(),
        sources: vec![g1_name],
        destinations: vec![g2_name],
    };
    let create_acl_policy = CreateAclPolicyRequest {
        acl_policy: AclPolicy {
            aclpolicy_id: id.to_string(),
            order: "0".to_string(),
            groups: vec![g1, g2],
            acls: vec![acl1],
        },
    };

    let _ignore = submit_policy_request(runtime_info, &create_acl_policy, &client).await;
    Ok(id.to_string())
}

pub async fn submit_policy_request(
    runtime_info: &RuntimeInformation,
    policy: &CreateAclPolicyRequest,
    client: &reqwest::Client,
) -> Result<String> {
    let policy_json = serde_json::to_string_pretty(&policy).expect("{}");
    println!("{}", policy_json);

    let res = client
        .post(format!(
            "{}/api/v1/aclpolicy",
            runtime_info.ninja_panda_api_url
        ))
        .bearer_auth(&runtime_info.ninja_panda_api_key)
        .json(&policy)
        .send()
        .await?;

    assert!(res.status().is_success(), "Could not create a policy");

    let answ = res.text().await?;
    dbg!(&answ);
    Ok(String::new())
}

pub async fn delete_acl_policy(
    runtime_info: &RuntimeInformation,
    client: &reqwest::Client,
    policy_id: &str,
) -> Result<String> {
    let res = client
        .delete(format!(
            "{}/api/v1/aclpolicy/{}",
            runtime_info.ninja_panda_api_url, policy_id
        ))
        .bearer_auth(&runtime_info.ninja_panda_api_key)
        .send()
        .await?;

    assert!(res.status().is_success(), "Could not delete policy");
    Ok(String::new())
}

pub async fn zero_out_acl_policy(
    runtime_info: &RuntimeInformation,
    client: &reqwest::Client,
    policy_id: &str,
) -> Result<String> {
    let blank_policy = UpdateAclPolicyRequest {
        acl_policies: vec![AclPolicy {
            aclpolicy_id: policy_id.to_string(),
            order: "0".to_string(),
            acls: vec![],
            groups: vec![],
        }],
    };
    let res = client
        .put(format!(
            "{}/api/v1/aclpolicy",
            runtime_info.ninja_panda_api_url
        ))
        .json(&blank_policy)
        .bearer_auth(&runtime_info.ninja_panda_api_key)
        .send()
        .await?;
    let text = res.text().await.unwrap();
    dbg!(text);
    // assert!(res.status().is_success(), "Could not zero-out policy");

    Ok(String::new())
}

/// Creates the namespace in NinjaPanda.  If the namespace already exists, then this does nothing.
pub async fn create_namespace(
    namespace_name: &str,
    runtime_info: &RuntimeInformation,
    client: &reqwest::Client,
) -> Result<()> {
    {
        let default_machine_key_ttl: String = "7777000000s".to_string();
        let ninja_panda_api_url: &str = &runtime_info.ninja_panda_api_url;
        let api_key: &str = &runtime_info.ninja_panda_api_key;
        async move {
            let create_namespace = CreateNamespaceRequest {
                name: namespace_name.to_owned(),
                default_machine_key_ttl,
            };
            let res = client
                .post(format!("{ninja_panda_api_url}/api/v1/namespace",))
                .bearer_auth(api_key)
                .json(&create_namespace)
                .send()
                .await?;
            let stri = res.text().await?;
            dbg!(stri);
            Ok(())
        }
    }
    .await
}

pub async fn delete_machine(
    machine_id: &str,
    runtime_info: &RuntimeInformation,
    client: &reqwest::Client,
) -> Result<()> {
    let fixed_machine_id = if machine_id.starts_with("machine:") {
        machine_id.replace("machine:", "")
    } else {
        machine_id.to_string()
    };
    let url = format!(
        "{}/api/v1/machine/{fixed_machine_id}",
        runtime_info.ninja_panda_api_url
    );
    let _res = client
        .delete(url)
        .bearer_auth(&runtime_info.ninja_panda_api_key)
        .send()
        .await?;
    Ok(())
}

pub async fn create_preauth_token(
    client: &reqwest::Client,
    runtime_information: &RuntimeInformation,
    request: CreatePreauthTokenRequest,
) -> Result<String> {
    let url = format!(
        "{}{}",
        runtime_information.ninja_panda_api_url, PREAUTH_TOKEN_API
    );

    let api_token = client
        .post(url)
        .json(&request)
        .bearer_auth(runtime_information.ninja_panda_api_key.as_str())
        .send()
        .await
        .unwrap();
    assert_eq!(200, api_token.status());

    let api_token_response: PreauthTokenResponse = api_token.json().await.unwrap();
    Ok(api_token_response.pre_auth_key.key)
}

pub async fn make_internet_gateway(
    runtime_info: &RuntimeInformation,
    client: &reqwest::Client,
    machine_id: &str,
) -> Result<()> {
    use crate::models::routes::{CreateRouteRequest, Route};

    let route_request = CreateRouteRequest {
        routes: vec![
            Route {
                prefix: "0.0.0.0/0".to_string(),
                enabled: true,
                advertised: true,
                is_primary: true,
                route_id: None,
                machine_id: None,
            },
            Route {
                prefix: "::/0".to_string(),
                enabled: true,
                advertised: true,
                is_primary: true,
                route_id: None,
                machine_id: None,
            },
        ],
    };
    let _res = client
        .post(format!(
            "{}/api/v1/machine/{}/routes",
            runtime_info.ninja_panda_api_url, machine_id,
        ))
        .bearer_auth(&runtime_info.ninja_panda_api_key)
        .json(&route_request)
        .send()
        .await?;
    Ok(())
}
