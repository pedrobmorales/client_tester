use rstest::{fixture, rstest};

mod equivalency_tests {
    use std::time::Duration;

    use super::*;
    use bollard::Docker;

    use reqwest::Client;

    use tokio::time::sleep;
    use ztclient_common::{
        containers::remove_container,
        errors::Errors,
        get_running_json,
        models::status::StatusResult,
        ninjapanda::create_namespace,
        random_container_name, start_and_register_client, start_and_register_client_nh,
        ztclient::{
            states::RUNNING_STATE, wait_for_state_change, ztclient_execute, ztclient_netmap,
        },
        Config, RuntimeInformation,
    };

    const NUM_CONTAINERS: u32 = 4;
    const SLEEP_TIME_MS: u64 = 800;

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
    async fn single_container_name_is_not_altered(
        runtime_info: RuntimeInformation,
        docker: Docker,
        config: Config,
        client: Client,
    ) {
        let mut error_container = Errors::new();
        let container_str = random_container_name();
        let container_name = container_str.as_str();
        let namespace_name = "scnamenot";
        create_namespace(namespace_name, &runtime_info, &client)
            .await
            .unwrap();

        start_and_register_client(
            &docker,
            &runtime_info,
            &config,
            &client,
            container_name,
            namespace_name,
            5,
        )
        .await;

        let _status = wait_for_state_change(&docker, container_name, RUNNING_STATE).await;

        // This is ugly, but hoping for a better netmap next time.
        let netmap = ztclient_netmap(&docker, container_name).await;
        error_container.string_eq_assert(
            format!("{}.{}.ztmesh.net", container_name, namespace_name),
            netmap.self_node.name,
        );

        remove_container(&docker, container_name).await
    }

    #[rstest]
    #[tokio::test]
    async fn node_equivalency_nopref(
        runtime_info: RuntimeInformation,
        docker: Docker,
        config: Config,
        client: Client,
    ) {
        let mut error_container = Errors::new();
        let container_str = random_container_name();
        let container_name = container_str.as_str();
        let namespace_name = "nodeeqnopref";
        create_namespace(namespace_name, &runtime_info, &client)
            .await
            .unwrap();

        for x in 1..NUM_CONTAINERS + 1 {
            start_and_register_client_nh(
                &docker,
                &runtime_info,
                &config,
                &client,
                container_name,
                namespace_name,
                x as usize % 9,
            )
            .await;
            if x > 1 {
                let _status = wait_for_state_change(&docker, container_name, RUNNING_STATE).await;

                // This is ugly, but hoping for a better netmap next time.
                let netmap = ztclient_netmap(&docker, container_name).await;
                error_container.string_eq_assert(
                    format!("{}-{}.{}.ztmesh.net", container_name, x - 1, namespace_name),
                    netmap.self_node.name,
                );
            }
            remove_container(&docker, container_name).await;
        }
    }

    #[rstest]
    #[tokio::test]
    async fn node_equivalency_withpref(
        runtime_info: RuntimeInformation,
        docker: Docker,
        config: Config,
        client: Client,
    ) {
        let mut error_container = Errors::new();
        let renamed_name = random_container_name();
        let namespace_name = "nodeeqwithpref";
        create_namespace(namespace_name, &runtime_info, &client)
            .await
            .unwrap();

        let mut container_names = Vec::new();

        for _ in 0..NUM_CONTAINERS {
            container_names.push(random_container_name());
        }

        let mut index = 1;
        for name in container_names.iter() {
            start_and_register_client(
                &docker,
                &runtime_info,
                &config,
                &client,
                name,
                namespace_name,
                4,
            )
            .await;

            wait_for_state_change(&docker, name, RUNNING_STATE).await;

            ztclient_execute(
                &docker,
                name,
                vec!["ztclient", "configure", "--hostname", renamed_name.as_str()],
            )
            .await
            .unwrap();

            ztclient_execute(&docker, name, vec!["ztclient", "disconnect"])
                .await
                .unwrap();
            ztclient_execute(&docker, name, vec!["ztclient", "connect"])
                .await
                .unwrap();

            sleep(Duration::from_millis(SLEEP_TIME_MS)).await;
            let _status: StatusResult = wait_for_state_change(&docker, name, RUNNING_STATE).await;
            let netmap = ztclient_netmap(&docker, name).await;
            dbg!(&_status.self_field.host_name, &netmap.self_node.name, index);

            if index > 1 {
                error_container.string_eq_assert(
                    format!(
                        "{}-{}.{}.ztmesh.net",
                        renamed_name,
                        index - 1,
                        namespace_name
                    ),
                    netmap.self_node.name,
                );
            }
            index += 1;

            remove_container(&docker, name).await;
        }
    }

    #[rstest]
    #[tokio::test]
    async fn capital_letters_in_hostname(
        runtime_info: RuntimeInformation,
        docker: Docker,
        config: Config,
        client: Client,
    ) {
        let mut error_container = Errors::new();
        let random_name = random_container_name().to_ascii_uppercase();
        let renamed_name = random_name.as_str();
        let namespace_name = "caplethostname";
        create_namespace(namespace_name, &runtime_info, &client)
            .await
            .unwrap();

        let mut container_names = Vec::new();

        for _ in 0..NUM_CONTAINERS {
            container_names.push(renamed_name)
        }

        let mut index = 1;
        for name in container_names.iter() {
            start_and_register_client_nh(
                &docker,
                &runtime_info,
                &config,
                &client,
                name,
                namespace_name,
                4,
            )
            .await;

            wait_for_state_change(&docker, name, RUNNING_STATE).await;

            sleep(Duration::from_millis(SLEEP_TIME_MS)).await;
            let _status: StatusResult = wait_for_state_change(&docker, name, RUNNING_STATE).await;
            let netmap = ztclient_netmap(&docker, name).await;
            dbg!(&_status.self_field.host_name, &netmap.self_node.name, index);

            if index > 1 {
                error_container.string_eq_assert(
                    format!(
                        "{}-{}.{}.ztmesh.net",
                        renamed_name,
                        index - 1,
                        namespace_name
                    )
                    .to_lowercase(),
                    netmap.self_node.name,
                );
            }
            index += 1;

            remove_container(&docker, renamed_name).await;
        }
    }

    #[rstest]
    #[tokio::test]
    async fn ending_in_numbers_hostname(
        runtime_info: RuntimeInformation,
        docker: Docker,
        config: Config,
        client: Client,
    ) {
        let mut error_collector = Errors::new();

        let random_name = format!("{}{}", random_container_name(), "1234");
        let renamed_name = random_name.as_str();
        let namespace_name = "endnumhostname";
        create_namespace(namespace_name, &runtime_info, &client)
            .await
            .unwrap();

        let mut container_names = Vec::new();

        for _ in 0..NUM_CONTAINERS {
            container_names.push(renamed_name)
        }

        let mut index = 1;
        for name in container_names.iter() {
            start_and_register_client_nh(
                &docker,
                &runtime_info,
                &config,
                &client,
                name,
                namespace_name,
                4,
            )
            .await;

            wait_for_state_change(&docker, name, RUNNING_STATE).await;

            sleep(Duration::from_millis(SLEEP_TIME_MS)).await;
            let status: StatusResult = wait_for_state_change(&docker, name, RUNNING_STATE).await;
            let netmap = ztclient_netmap(&docker, name).await;
            error_collector.expect_none(&netmap.packet_filter, "netmap.packet_filter");
            dbg!(&status.self_field.host_name, &netmap.self_node.name, index);

            if index > 1 {
                error_collector.string_eq_assert(
                    format!(
                        "{}-{}.{}.ztmesh.net",
                        renamed_name,
                        index - 1,
                        namespace_name
                    )
                    .to_lowercase(),
                    netmap.self_node.name,
                );
                error_collector.string_eq_assert(
                    status.self_field.host_name,
                    format!("{}-{}", renamed_name, index - 1),
                );
            }
            index += 1;

            remove_container(&docker, renamed_name).await;
        }

        error_collector.assert_pop();
    }

    #[rstest]
    #[tokio::test]
    async fn with_underscores_in_hostname(
        runtime_info: RuntimeInformation,
        docker: Docker,
        config: Config,
        client: Client,
    ) {
        let mut error_container = Errors::new();
        let random_name = format!("a_{}", random_container_name());
        let renamed_name = random_name.replace('_', "-");
        let renamed_name = renamed_name.as_str();
        let namespace_name = "withundhostname";

        create_namespace(namespace_name, &runtime_info, &client)
            .await
            .unwrap();

        let mut container_names = Vec::new();

        for _ in 0..NUM_CONTAINERS {
            container_names.push(random_name.clone())
        }

        let mut index = 1;

        for name in container_names.iter() {
            start_and_register_client_nh(
                &docker,
                &runtime_info,
                &config,
                &client,
                name,
                namespace_name,
                4,
            )
            .await;

            wait_for_state_change(&docker, name, RUNNING_STATE).await;

            sleep(Duration::from_millis(SLEEP_TIME_MS)).await;
            let _status: StatusResult = wait_for_state_change(&docker, name, RUNNING_STATE).await;
            let netmap = ztclient_netmap(&docker, name).await;
            dbg!(&_status.self_field.host_name, &netmap.self_node.name, index);

            if index > 1 {
                error_container.string_eq_assert(
                    format!(
                        "{}-{}.{}.ztmesh.net",
                        renamed_name,
                        index - 1,
                        namespace_name
                    )
                    .to_lowercase(),
                    netmap.self_node.name,
                );
            }
            index += 1;

            remove_container(&docker, name).await;
        }
    }

    #[rstest]
    #[tokio::test]
    async fn with_hyphens_in_hostname(
        runtime_info: RuntimeInformation,
        docker: Docker,
        config: Config,
        client: Client,
    ) {
        let mut error_container = Errors::new();
        let random_name = format!("a-{}-b", random_container_name());
        let renamed_name = random_name.as_str();
        let namespace_name = "withhyphhostname";

        create_namespace(namespace_name, &runtime_info, &client)
            .await
            .unwrap();

        let mut container_names = Vec::new();

        for _ in 0..NUM_CONTAINERS {
            container_names.push(renamed_name)
        }

        let mut index = 1;
        for name in container_names.iter() {
            start_and_register_client_nh(
                &docker,
                &runtime_info,
                &config,
                &client,
                name,
                namespace_name,
                4,
            )
            .await;

            wait_for_state_change(&docker, name, RUNNING_STATE).await;

            sleep(Duration::from_millis(SLEEP_TIME_MS)).await;
            let _status: StatusResult = wait_for_state_change(&docker, name, RUNNING_STATE).await;
            let netmap = ztclient_netmap(&docker, name).await;
            dbg!(&_status.self_field.host_name, &netmap.self_node.name, index);

            if index > 1 {
                error_container.string_eq_assert(
                    format!(
                        "{}-{}.{}.ztmesh.net",
                        renamed_name,
                        index - 1,
                        namespace_name
                    )
                    .to_lowercase(),
                    netmap.self_node.name,
                );
            }
            index += 1;

            remove_container(&docker, renamed_name).await;
        }
    }

    #[rstest]
    #[tokio::test]
    async fn capital_letters_in_connhnpref(
        runtime_info: RuntimeInformation,
        docker: Docker,
        config: Config,
        client: Client,
    ) {
        let mut error_container = Errors::new();
        let random_name = random_container_name().to_ascii_uppercase();
        let renamed_name = random_name.as_str();
        let namespace_name = "caplethnpref";
        create_namespace(namespace_name, &runtime_info, &client)
            .await
            .unwrap();

        let mut container_names = Vec::new();

        for _ in 0..NUM_CONTAINERS {
            container_names.push(renamed_name)
        }

        let mut index = 1;
        for name in container_names.iter() {
            start_and_register_client(
                &docker,
                &runtime_info,
                &config,
                &client,
                name,
                namespace_name,
                4,
            )
            .await;

            wait_for_state_change(&docker, name, RUNNING_STATE).await;

            sleep(Duration::from_millis(SLEEP_TIME_MS)).await;
            let _status: StatusResult = wait_for_state_change(&docker, name, RUNNING_STATE).await;
            let netmap = ztclient_netmap(&docker, name).await;
            dbg!(&_status.self_field.host_name, &netmap.self_node.name, index);

            if index > 1 {
                error_container.string_eq_assert(
                    format!(
                        "{}-{}.{}.ztmesh.net",
                        renamed_name,
                        index - 1,
                        namespace_name
                    )
                    .to_lowercase(),
                    netmap.self_node.name,
                );
            }
            index += 1;

            remove_container(&docker, renamed_name).await;
        }
    }

    #[rstest]
    #[tokio::test]
    async fn ending_in_numbers_in_connhnpref(
        runtime_info: RuntimeInformation,
        docker: Docker,
        config: Config,
        client: Client,
    ) {
        let mut error_container = Errors::new();
        let random_name = random_container_name();
        let random_name = format!("{}{}", random_name, "1234");
        let renamed_name = random_name.as_str();
        let namespace_name = "endnumhnpref";
        create_namespace(namespace_name, &runtime_info, &client)
            .await
            .unwrap();

        let mut container_names = Vec::new();

        for _ in 0..NUM_CONTAINERS {
            container_names.push(renamed_name)
        }

        let mut index = 1;
        for name in container_names.iter() {
            start_and_register_client(
                &docker,
                &runtime_info,
                &config,
                &client,
                name,
                namespace_name,
                4,
            )
            .await;

            wait_for_state_change(&docker, name, RUNNING_STATE).await;

            sleep(Duration::from_millis(SLEEP_TIME_MS)).await;
            let _status: StatusResult = wait_for_state_change(&docker, name, RUNNING_STATE).await;
            let netmap = ztclient_netmap(&docker, name).await;
            dbg!(&_status.self_field.host_name, &netmap.self_node.name, index);

            if index > 1 {
                error_container.string_eq_assert(
                    format!(
                        "{}-{}.{}.ztmesh.net",
                        renamed_name,
                        index - 1,
                        namespace_name
                    )
                    .to_lowercase(),
                    netmap.self_node.name,
                );
            }
            index += 1;

            remove_container(&docker, renamed_name).await;
        }
    }

    #[rstest]
    #[tokio::test]
    async fn with_underscores_in_connhnpref(
        runtime_info: RuntimeInformation,
        docker: Docker,
        config: Config,
        client: Client,
    ) {
        let mut error_container = Errors::new();
        let random_name = random_container_name();
        let random_name = format!("a_{}", random_name);
        let renamed_name = random_name.replace('_', "-");
        let renamed_name = renamed_name.as_str();
        let namespace_name = "withundhnpref";

        create_namespace(namespace_name, &runtime_info, &client)
            .await
            .unwrap();

        let mut container_names = Vec::new();

        for _ in 0..NUM_CONTAINERS {
            container_names.push(random_name.clone())
        }

        let mut index = 1;

        for name in container_names.iter() {
            start_and_register_client(
                &docker,
                &runtime_info,
                &config,
                &client,
                name,
                namespace_name,
                4,
            )
            .await;

            wait_for_state_change(&docker, name, RUNNING_STATE).await;

            sleep(Duration::from_millis(SLEEP_TIME_MS)).await;
            let _status: StatusResult = wait_for_state_change(&docker, name, RUNNING_STATE).await;
            let netmap = ztclient_netmap(&docker, name).await;
            dbg!(&_status.self_field.host_name, &netmap.self_node.name, index);

            if index > 1 {
                error_container.string_eq_assert(
                    format!(
                        "{}-{}.{}.ztmesh.net",
                        renamed_name,
                        index - 1,
                        namespace_name
                    )
                    .to_lowercase(),
                    netmap.self_node.name,
                );
            }
            index += 1;

            remove_container(&docker, name).await;
        }
    }

    #[rstest]
    #[tokio::test]
    async fn with_hyphens_in_connhnpref(
        runtime_info: RuntimeInformation,
        docker: Docker,
        config: Config,
        client: Client,
    ) {
        let random_name = random_container_name();
        let random_name = format!("a-{}-b", random_name);
        let renamed_name = random_name.as_str();
        let namespace_name = "withhyphhnpref";

        create_namespace(namespace_name, &runtime_info, &client)
            .await
            .unwrap();

        let mut container_names = Vec::new();

        for _ in 0..NUM_CONTAINERS {
            container_names.push(renamed_name)
        }

        let mut index = 1;
        for name in container_names.iter() {
            start_and_register_client(
                &docker,
                &runtime_info,
                &config,
                &client,
                name,
                namespace_name,
                4,
            )
            .await;

            wait_for_state_change(&docker, name, RUNNING_STATE).await;

            sleep(Duration::from_millis(SLEEP_TIME_MS)).await;
            let _status: StatusResult = wait_for_state_change(&docker, name, RUNNING_STATE).await;
            let netmap = ztclient_netmap(&docker, name).await;
            dbg!(&_status.self_field.host_name, &netmap.self_node.name, index);

            if index > 1 {
                assert_eq!(
                    format!(
                        "{}-{}.{}.ztmesh.net",
                        renamed_name,
                        index - 1,
                        namespace_name
                    )
                    .to_lowercase(),
                    netmap.self_node.name,
                );
            }
            index += 1;

            remove_container(&docker, renamed_name).await;
        }
    }

    #[rstest]
    #[tokio::test]
    async fn multiple_renames(
        runtime_info: RuntimeInformation,
        docker: Docker,
        config: Config,
        client: Client,
    ) {
        let random_name = random_container_name();
        let namespace_name = "multirens";

        create_namespace(namespace_name, &runtime_info, &client)
            .await
            .unwrap();

        start_and_register_client(
            &docker,
            &runtime_info,
            &config,
            &client,
            &random_name,
            namespace_name,
            4,
        )
        .await;

        wait_for_state_change(&docker, &random_name, RUNNING_STATE).await;

        sleep(Duration::from_millis(SLEEP_TIME_MS)).await;
        let _status: StatusResult =
            wait_for_state_change(&docker, &random_name, RUNNING_STATE).await;
        let netmap = ztclient_netmap(&docker, &random_name).await;
        dbg!(&_status.self_field.host_name, &netmap.self_node.name);

        for index in 0..NUM_CONTAINERS {
            let new_name = random_container_name();
            ztclient_execute(
                &docker,
                &random_name,
                vec!["ztclient", "configure", "--hostname", new_name.as_str()],
            )
            .await
            .unwrap();

            ztclient_execute(&docker, &random_name, vec!["ztclient", "disconnect"])
                .await
                .unwrap();
            ztclient_execute(&docker, &random_name, vec!["ztclient", "connect"])
                .await
                .unwrap();

            sleep(Duration::from_millis(SLEEP_TIME_MS)).await;
            let _status: StatusResult =
                wait_for_state_change(&docker, &random_name, RUNNING_STATE).await;
            let netmap = ztclient_netmap(&docker, &random_name).await;
            dbg!(&_status.self_field.host_name, &netmap.self_node.name, index);
        }
        remove_container(&docker, &random_name).await;
    }
}
