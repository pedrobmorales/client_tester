use rstest::{fixture, rstest};

mod machine_delete_tests {
    use super::*;
    use bollard::Docker;

    use reqwest::Client;

    use ztclient_common::{
        execute_callback, get_running_json,
        ninjapanda::{
            create_namespace, delete_machine, get_all_machine_ids, make_all_machines_peers,
        },
        random_container_name,
        users::get_user,
        ztclient::{
            start_ztclientd, states::RUNNING_STATE, wait_for_state_change, ztclient_registration,
        },
        Config, ExecuteCallbackRequest, RuntimeInformation,
    };

    #[fixture]
    fn runtime_info() -> RuntimeInformation {
        get_running_json().expect("Unable to read runtime.json, is environment created?")
    }

    #[fixture]
    fn docker() -> Docker {
        Docker::connect_with_unix_defaults().unwrap()
    }

    #[fixture]
    fn config() -> Config {
        dotenv::dotenv().ok();

        match envy::from_env::<Config>() {
            Ok(config) => config,
            Err(error) => panic!("{:#?}", error),
        }
    }

    #[fixture]
    fn client() -> reqwest::Client {
        reqwest::Client::new()
    }

    #[rstest]
    #[tokio::test]
    async fn test_delete_one_peer(
        runtime_info: RuntimeInformation,
        docker: Docker,
        config: Config,
        client: Client,
    ) {
        let num_clients = 4;
        let hostname_prefix = random_container_name();
        // Create the namespace after NGINX has started, because if we are clustered, NGINX is the way to reach NP.
        let namespace_name = "mdel01";
        create_namespace(namespace_name, &runtime_info, &client)
            .await
            .unwrap();

        for x in 1..num_clients + 1 {
            let name = format!("{}{:0>3}", hostname_prefix, x);
            start_ztclientd(&docker, &config, name.as_str())
                .await
                .unwrap();
            let correlation_id = ztclient_registration(&docker, name.as_str()).await.unwrap();
            let request = ExecuteCallbackRequest {
                correlation_id: correlation_id.as_str(),
                api_key: &runtime_info.ninja_panda_api_key,
                namespace_name,
                user_info_id: (x as usize % 9),
                ninja_panda_api_url: &runtime_info.ninja_panda_api_url,
            };
            execute_callback(&client, &request).await.unwrap();
        }

        // Check that the userInfo is correct for newly created nodes that have no peers.
        for x in 1..num_clients + 1 {
            let name = format!("{}{:0>3}", hostname_prefix, x);
            let user_info = get_user(x as usize % 9);

            let status = wait_for_state_change(&docker, name.as_str(), RUNNING_STATE).await;
            let assigned_user_id = status.self_field.user_id;

            let user_map = status.user.unwrap();
            let user_object = user_map.get(&assigned_user_id.to_string()).unwrap();
            user_object.assert_eq(&user_info);
        }

        let machine_ids = get_all_machine_ids(&runtime_info, &client, &[hostname_prefix.clone()])
            .await
            .unwrap();
        let policy_id = make_all_machines_peers(&runtime_info, &machine_ids, &client)
            .await
            .unwrap();

        dbg!(&policy_id);
        // Now let's check the user names after the peers have been declared.

        // For hosts where the docker container name matches the hostname.
        for x in 1..num_clients + 1 {
            let name = format!("{}{:0>3}", hostname_prefix, x);
            let user_info = get_user(x as usize % 9);

            let status = wait_for_state_change(&docker, name.as_str(), RUNNING_STATE).await;
            let assigned_user_id = status.self_field.user_id;

            let user_map = status.user.unwrap();
            let user_object = user_map.get(&assigned_user_id.to_string()).unwrap();
            user_object.assert_eq(&user_info);

            // remove_container(&docker, &name).await;
        }

        let machine_id = machine_ids.get(1).unwrap();
        dbg!(&machine_id);
        delete_machine(machine_id, &runtime_info, &client)
            .await
            .unwrap();
    }
}
