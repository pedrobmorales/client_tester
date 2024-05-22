/**
 * Test cases:
 * 1.  Create two machines, make them one-way peers on port 80, A->B.
 *     - A.peers={B}
 *
 * 2.  Create two machines, make them two-way peers on port 80, A->B and B->A
 * 3.  Create three machines, A, B and C.  Make A->B (port 80) peers, and B->C port 75 peers
 */
mod policies_tests {
    use std::time::Duration;

    use anyhow::Result;
    use bollard::Docker;
    use rstest::{fixture, rstest};
    use tokio::time::sleep;
    use ztclient_common::{
        containers::{remove_container, ContainerRemover},
        errors::Errors,
        get_running_json, get_unique_timestamp,
        ninjapanda::{
            create_namespace, delete_acl_policy, get_all_machine_ids, grant_one_directional_policy,
            make_all_machines_peers, zero_out_acl_policy,
        },
        random_container_name,
        ztclient::{create_and_register_client, create_running_clients, ztclient_netmap},
        Config, RuntimeInformation,
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
    async fn test01(
        runtime_info: RuntimeInformation,
        docker: Docker,
        config: Config,
        client: reqwest::Client,
    ) -> Result<()> {
        // * 1.  Create two machines, make them one-way peers on port 80, A->B.
        // *     - A.peers={B}
        let mut error_container = Errors::new();
        let namespace_suffix = get_unique_timestamp();
        let namespace_name = format!("optm{namespace_suffix}");
        create_namespace(namespace_name.as_str(), &runtime_info, &client)
            .await
            .unwrap();
        let container_name1 = "policy01";
        let container_name2 = "policy02";
        let user_info_id = 3;
        let machine_id1 = create_and_register_client(
            &runtime_info,
            &docker,
            &config,
            &client,
            container_name1,
            namespace_name.as_str(),
            user_info_id,
        )
        .await
        .unwrap();

        let machine_id2 = create_and_register_client(
            &runtime_info,
            &docker,
            &config,
            &client,
            container_name2,
            namespace_name.as_str(),
            user_info_id,
        )
        .await
        .unwrap();

        // Make them one-way peers
        let policy_id =
            grant_one_directional_policy(&runtime_info, &machine_id1, &machine_id2, 80, &client)
                .await?;

        sleep(Duration::from_millis(5000)).await;

        let netmap1 = ztclient_netmap(&docker, &container_name1).await;
        error_container.expect_some(&netmap1.peers, "NetMap1.Peers");
        error_container.expect_none(&netmap1.packet_filter, "NetMap1.PacketFilter");

        let netmap2 = ztclient_netmap(&docker, &container_name2).await;
        error_container.expect_some(&netmap2.peers, "NetMap2.Peers");
        error_container.expect_some(&netmap2.packet_filter, "NetMap2.PacketFilter");

        zero_out_acl_policy(&runtime_info, &client, &policy_id).await?;
        sleep(Duration::from_millis(5000)).await;

        let netmap1_1 = ztclient_netmap(&docker, &container_name1).await;
        error_container.expect_none(&netmap1_1.peers, "NetMap1_1.Peers");

        // remove_container(&docker, container_name1).await;
        // remove_container(&docker, container_name2).await;

        error_container.assert_pop();

        Ok(())
    }
}
