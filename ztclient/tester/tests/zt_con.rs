use rstest::{fixture, rstest};
mod zt_con_tests {

    use super::*;
    use bollard::{
        container::LogOutput,
        exec::{CreateExecOptions, StartExecResults},
        Docker,
    };

    use futures::StreamExt;

    use ztclient_common::{
        containers::ContainerRemover,
        errors::Errors,
        get_running_json, get_unique_timestamp,
        models::ztn::SelfNode,
        ninjapanda::{create_namespace, grant_one_directional_policy},
        ztclient::{create_and_register_client, wait_for_peer, ztclient_netmap},
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
    async fn wait_peer_ztcon(_docker: Docker, _config: Config) {
        let _container_name = "ztclienthost004";
    }

    #[rstest]
    #[tokio::test]
    async fn wait_for_policy(
        runtime_info: RuntimeInformation,
        docker: Docker,
        config: Config,
        client: reqwest::Client,
    ) {
        let dummy_node = SelfNode::default();

        let mut error_container = Errors::new();
        let namespace_suffix = get_unique_timestamp();
        let namespace_name = format!("optm{namespace_suffix}");
        create_namespace(namespace_name.as_str(), &runtime_info, &client)
            .await
            .unwrap();

        // Create container1 and container2
        let container_name1 = "wfpolicy01";
        let container_name2 = "wfpolicy02";
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

        let _dropper1 = ContainerRemover::new(container_name1.to_string());
        let _dropper2 = ContainerRemover::new(container_name2.to_string());

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

        println!("Machines created, sleeping!");
        // sleep(Duration::from_millis(1300)).await;

        let _policy_id =
            grant_one_directional_policy(&runtime_info, &machine_id1, &machine_id2, 80, &client)
                .await
                .unwrap();
        println!("Policy created, sleeping!");
        // sleep(Duration::from_millis(1300)).await;

        println!("Continuing on with wait for peer!");
        // sleep(Duration::from_secs(20)).await;
        // Let's execute a Wait For Peer
        let result = wait_for_peer(&docker, container_name1, container_name2)
            .await
            .unwrap();
        dbg!(result);

        // sleep(Duration::from_secs(15)).await;

        // Let's verify the netmaps
        let netmap1 = ztclient_netmap(&docker, container_name1).await;
        error_container.expect_none(
            &netmap1.packet_filter,
            "First container should not have a packet filter",
        );
        error_container.expect_some(&netmap1.peers, "Node1.Peers");
        let peers = netmap1.peers.unwrap_or_default();
        error_container.num_eq_assert(1, peers.len(), "Node expected to have exactly one peer");
        error_container.expect_some(&netmap1.packet_filter, "Node1.PacketFilter");
        let other_peer = peers.first().unwrap_or(&dummy_node);
        dbg!(&other_peer.session_key);

        let netmap2 = ztclient_netmap(&docker, container_name2).await;
        error_container.expect_some(
            &netmap2.packet_filter,
            "Second container should not have a packet filter",
        );
        error_container.expect_none(&netmap2.peers, "Node2.Peers");
        println!("Executing first zt-con!");
        let response = ztcon(&docker, container_name1, container_name2, 80).await;
        error_container.string_eq_assert(response, "HELLO".to_string());

        println!("Executing second zt-con!");
        let response = ztcon(&docker, container_name1, container_name2, 80).await;
        error_container.string_eq_assert(response, "HELLO".to_string());

        error_container.assert_pop();
        // remove_container(&docker, container_name1).await;
        // remove_container(&docker, container_name2).await;
    }

    pub async fn ztcon(docker: &Docker, client1: &str, client2: &str, port_number: u32) -> String {
        let res = docker
            .create_exec(
                client1,
                CreateExecOptions {
                    cmd: Some(vec![
                        "ztclient",
                        "zt-con",
                        client2,
                        &port_number.to_string(),
                    ]),
                    attach_stderr: Some(true),
                    attach_stdin: Some(true),
                    attach_stdout: Some(true),
                    ..Default::default()
                },
            )
            .await;
        let result = res.unwrap();
        let output_result = docker.start_exec(result.id.as_str(), None).await.unwrap();

        let mut vect = Vec::new();
        match output_result {
            StartExecResults::Attached {
                mut output,
                mut input,
            } => {
                let output_line = output.next().await;
                match output_line {
                    Some(Ok(LogOutput::StdOut { message })) => {
                        dbg!("StdOut", &message);
                        vect.push(message);
                        ""
                    }
                    Some(Ok(LogOutput::StdErr { message })) => {
                        dbg!("StdErr", &message);
                        vect.push(message);
                        ""
                    }
                    _ => "",
                };
            }
            StartExecResults::Detached => (),
        };
        vect.first()
            .map(|x| std::str::from_utf8(x))
            .filter(|x| x.is_ok())
            .map(|x| x.unwrap())
            .unwrap()
            .to_string()
    }
}
