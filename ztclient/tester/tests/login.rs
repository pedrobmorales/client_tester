use rstest::{fixture, rstest};

mod login_logout_tests {

    use super::*;
    use bollard::Docker;

    use reqwest::Client;

    use ztclient_common::{
        containers::remove_container,
        errors::Errors,
        execute_callback, get_running_json,
        models::status::StatusResult,
        ninjapanda::{
            create_namespace, delete_machine, get_all_machine_ids, make_all_machines_peers,
        },
        random_container_name,
        users::get_user,
        ztclient::{
            create_and_register_client, start_ztclientd,
            states::{NEEDS_LOGIN_STATE, RUNNING_STATE},
            wait_for_state_change, ztclient_alternate_hostname_registration, ztclient_logout,
            ztclient_registration, ztclient_status_json,
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
    async fn create_basic_setup(
        runtime_info: RuntimeInformation,
        docker: Docker,
        config: Config,
        client: Client,
    ) {
        let mut error_container = Errors::new();
        let num_clients = 2;
        let total_peers = num_clients * 2 - 1;
        let hostname_prefix = random_container_name();
        // Create the namespace after NGINX has started, because if we are clustered, NGINX is the way to reach NP.
        let namespace_name = "loginoutbs";

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
                user_info_id: (x % 9),
                ninja_panda_api_url: &runtime_info.ninja_panda_api_url,
            };
            execute_callback(&client, &request).await.unwrap();
        }

        // Check that the userInfo is correct for newly created nodes that have no peers.
        for x in 1..num_clients + 1 {
            let name = format!("{}{:0>3}", hostname_prefix, x);
            let user_info = get_user(x % 9);

            let status = wait_for_state_change(&docker, name.as_str(), RUNNING_STATE).await;
            let assigned_user_id = status.self_field.user_id;

            let user_map = status.user.unwrap();
            let user_object = user_map.get(&assigned_user_id.to_string()).unwrap();
            error_container.verify_user(user_object, &user_info);
        }

        let random_container_name2 = random_container_name();
        let random_container_name3 = random_container_name();
        for x in 1..num_clients + 1 {
            let name = format!("{}{:0>3}", random_container_name2, x);
            let hostname = format!("{}{:0>3}", random_container_name3, x);
            start_ztclientd(&docker, &config, name.as_str())
                .await
                .unwrap();
            let correlation_id =
                ztclient_alternate_hostname_registration(&docker, name.as_str(), hostname.as_str())
                    .await
                    .unwrap();
            let request = ExecuteCallbackRequest {
                correlation_id: correlation_id.as_str(),
                api_key: &runtime_info.ninja_panda_api_key,
                namespace_name,
                user_info_id: (x % 9),
                ninja_panda_api_url: &runtime_info.ninja_panda_api_url,
            };
            execute_callback(&client, &request).await.unwrap();
        }

        // Check that the userInfo is correct for newly created nodes that have no peers.
        for x in 1..num_clients + 1 {
            let name = format!("{}{:0>3}", random_container_name2, x);
            let user_info = get_user(x % 9);

            let status = wait_for_state_change(&docker, name.as_str(), RUNNING_STATE).await;
            let assigned_user_id = status.self_field.user_id;

            let user_map = status.user.unwrap();
            let user_object = user_map.get(&assigned_user_id.to_string()).unwrap();
            error_container.verify_user(user_object, &user_info);
        }

        let machine_names = vec![
            hostname_prefix.to_string(),
            random_container_name3.to_string(),
        ];
        let machine_ids = get_all_machine_ids(&runtime_info, &client, &machine_names)
            .await
            .unwrap();
        let _policy_id = make_all_machines_peers(&runtime_info, &machine_ids, &client)
            .await
            .unwrap();

        // Now let's check the user names after the peers have been declared.

        // For hosts where the docker container name matches the hostname.
        for x in 1..num_clients + 1 {
            let name = format!("{}{:0>3}", hostname_prefix, x);
            let user_info = get_user(x % 9);

            let status = wait_for_state_change(&docker, name.as_str(), RUNNING_STATE).await;
            let assigned_user_id = status.self_field.user_id;

            let user_map = status.user.unwrap();
            let user_object = user_map.get(&assigned_user_id.to_string()).unwrap();
            error_container.verify_user(user_object, &user_info);

            remove_container(&docker, &name).await;
        }

        // For hosts where the docker container name does not match the hostname.
        for x in 1..num_clients + 1 {
            let name = format!("{}{:0>3}", random_container_name2, x);
            let hostname = format!("{}{:0>3}", random_container_name3, x);
            let user_info = get_user(x % 9);

            let status = ztclient_status_json(&docker, name.as_str()).await.unwrap();
            let assigned_user_id = status.self_field.user_id;

            let user_map = status.user.unwrap();
            let user_object = user_map.get(&assigned_user_id.to_string()).unwrap();
            user_object.assert_eq(&user_info);
            error_container.string_eq_assert(hostname, status.self_field.host_name);
            let peers = status.peer.unwrap_or_default();
            error_container.num_eq_assert(peers.len(), total_peers, "Unexpected number of clients");

            remove_container(&docker, &name).await;
        }
    }

    #[rstest]
    #[tokio::test]
    async fn login_logout(
        runtime_info: RuntimeInformation,
        docker: Docker,
        config: Config,
        client: Client,
    ) {
        let _error_container = Errors::new();
        let container_name = random_container_name();
        let container_name = &container_name;
        let namespace_name = random_container_name();
        let user_id = 5;

        create_namespace(&namespace_name, &runtime_info, &client)
            .await
            .unwrap();

        let machine_id = create_and_register_client(
            &runtime_info,
            &docker,
            &config,
            &client,
            container_name,
            &namespace_name,
            user_id,
        )
        .await
        .unwrap();

        let _status = wait_for_state_change(&docker, container_name, RUNNING_STATE).await;
        ztclient_logout(&docker, container_name).await.unwrap();
        let _status = wait_for_state_change(&docker, container_name, NEEDS_LOGIN_STATE).await;

        delete_machine(&machine_id, &runtime_info, &client)
            .await
            .unwrap();

        remove_container(&docker, container_name).await;
    }

    #[rstest]
    #[tokio::test]
    async fn change_user_name(
        runtime_info: RuntimeInformation,
        docker: Docker,
        config: Config,
        client: Client,
    ) {
        let _error_container = Errors::new();
        let container_name_str = random_container_name();
        let container_name = container_name_str.as_str();
        let namespace_name = random_container_name();
        let user_id = 5;

        create_namespace(&namespace_name, &runtime_info, &client)
            .await
            .unwrap();

        let _machine_id = create_and_register_client(
            &runtime_info,
            &docker,
            &config,
            &client,
            container_name,
            &namespace_name,
            user_id,
        )
        .await
        .unwrap();

        let status: StatusResult =
            wait_for_state_change(&docker, container_name, RUNNING_STATE).await;
        assert_eq!(RUNNING_STATE, status.backend_state);

        ztclient_logout(&docker, container_name).await.unwrap();

        let status = ztclient_status_json(&docker, container_name).await.unwrap();
        assert_eq!(NEEDS_LOGIN_STATE, status.backend_state);

        let user_id = 2;

        let correlation_id = ztclient_registration(&docker, container_name)
            .await
            .unwrap();
        let request = ExecuteCallbackRequest {
            correlation_id: correlation_id.as_str(),
            api_key: &runtime_info.ninja_panda_api_key,
            namespace_name: &namespace_name,
            user_info_id: user_id,
            ninja_panda_api_url: &runtime_info.ninja_panda_api_url,
        };
        let machine_id = execute_callback(&client, &request).await.unwrap();

        let status: StatusResult =
            wait_for_state_change(&docker, container_name, RUNNING_STATE).await;

        if let Some(users) = status.user {
            if let Some(user) = users.get("2") {
                assert_eq!("user02@optm.com", user.login_name);
            }
        } else {
            assert!(false, "Status did not return any users");
        }

        delete_machine(&machine_id, &runtime_info, &client)
            .await
            .unwrap();

        remove_container(&docker, container_name).await;
    }
}
