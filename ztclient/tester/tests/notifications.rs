use rstest::{fixture, rstest};

mod notifications_tests {
    use std::time::Duration;

    use super::*;

    use bollard::Docker;

    use reqwest::Client;

    use tokio::time::sleep;
    use ztclient_common::{
        container_cleanup,
        errors::Errors,
        get_running_json,
        ninjapanda::{get_all_machine_ids, make_all_machines_peers, make_internet_gateway},
        random_container_name,
        ztclient::{create_running_clients, ztclient_netmap},
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
    async fn notify_setup(
        runtime_info: RuntimeInformation,
        docker: Docker,
        config: Config,
        client: Client,
    ) {
        let mut error_container = Errors::new();
        let hostname_prefix: String = random_container_name();
        let num_clients = 4;
        let namespace_name = "notifs";
        let container_names = create_running_clients(
            &runtime_info,
            &docker,
            &config,
            &client,
            hostname_prefix,
            num_clients,
            namespace_name,
        )
        .await;

        let machine_ids = get_all_machine_ids(&runtime_info, &client, &container_names)
            .await
            .unwrap();
        let policy_id = make_all_machines_peers(&runtime_info, &machine_ids, &client)
            .await
            .unwrap();

        dbg!(&policy_id);

        // Make one of them an internet gateway
        let machine_id1 = machine_ids.first().unwrap();
        let machine_id1 = &machine_id1.replace("machine:", "");
        make_internet_gateway(&runtime_info, &client, machine_id1)
            .await
            .unwrap();

        let mut stop_loop = false;
        let mut counter = 0;
        while !stop_loop {
            // Now let's check the user names after the peers have been declared.
            stop_loop = true;
            counter += 1;
            for x in container_names.iter() {
                // dbg!(x);
                let netmap = ztclient_netmap(&docker, x).await;
                let peers = netmap.peers.unwrap_or_default();
                if peers.is_empty() {
                    stop_loop = false;
                    sleep(Duration::from_secs(5)).await;
                    continue;
                }
                error_container.num_eq_assert(
                    num_clients - 1,
                    peers.len(),
                    format!("Unexpected number of peers for {}", netmap.self_node.name).as_str(),
                );
                let _packet_filter = netmap.packet_filter.unwrap_or_default();

                // Assert that all the peers are online
                for peer in peers.iter() {
                    error_container.bool_assert(
                        peer.online,
                        format!(
                            "For machine={} peer {} not online but expected to!",
                            netmap.self_node.name, peer.name
                        ),
                    );
                }
            }
            if counter > 15 {
                break;
            }
        }
        dbg!(counter);
        let ig_route4 = "0.0.0.0/0".to_string();
        let mut count = 0;
        for x in container_names.iter() {
            let netmap = ztclient_netmap(&docker, x).await;
            let self_node = netmap.self_node;
            let packet_filter = netmap.packet_filter.unwrap_or_default();
            dbg!(&packet_filter);
            if self_node.allowed_ips.contains(&ig_route4) {
                // If it contains the IG route it must also have it in PrimaryIPs
                let primary_routes = self_node.primary_routes.unwrap_or_default();
                dbg!(&primary_routes);
                count += 1;
            }
        }

        dbg!("Number of internet gateways", count);
        container_cleanup(
            &docker,
            container_names,
            machine_ids,
            &runtime_info,
            &client,
        )
        .await;
        error_container.assert_pop();
    }
}
