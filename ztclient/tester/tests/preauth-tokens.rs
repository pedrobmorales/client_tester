use rstest::{fixture, rstest};
mod preauth_tokens_tests {
    use super::*;
    use bollard::Docker;

    use ztclient_common::{
        containers::remove_container,
        execute_callback, get_running_json,
        models::CreatePreauthTokenRequest,
        ninjapanda::{create_namespace, create_preauth_token},
        users::get_user,
        ztclient::{
            create_client_with_preauth_token, preauth_token_registration, states::RUNNING_STATE,
            wait_for_state_change, ztclient_execute, ztclient_logout,
            ztclient_register_forcereauth, ztclient_registration, NGINX_NP_URL,
        },
        Config, ExecuteCallbackRequest, RuntimeInformation,
    };

    const INVALID_AUTH_TOKEN_ERROR: &str = "backend error: Invalid preauth token\n";

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

    async fn login_with_valid_preauth_token(
        config: Config,
        docker: Docker,
        client: reqwest::Client,
        runtime_info: RuntimeInformation,
    ) {
        let namespace_name = "optm";
        create_namespace(namespace_name, &runtime_info, &client)
            .await
            .unwrap();

        let preauth_token = create_preauth_token(
            &client,
            &runtime_info,
            CreatePreauthTokenRequest {
                acl_tags: vec![],
                namespace: namespace_name.to_string(),
                prefix: "".to_string(),
                reuse_count: 0,
                ephemeral: false,
                expiration: "".to_string(),
            },
        )
        .await
        .unwrap();

        let result = create_client_with_preauth_token(
            &docker,
            &config,
            "preauth_token_test01",
            preauth_token.as_str(),
        )
        .await
        .unwrap();
        similar_asserts::assert_eq!("", result);
        remove_container(&docker, "preauth_token_test01").await;
    }

    #[rstest]
    #[tokio::test]
    async fn login_with_invalid_preauth_token(config: Config, docker: Docker) {
        let preauth_token = "asjhfasjkfhalskjfhasljfkas";
        let result = create_client_with_preauth_token(
            &docker,
            &config,
            "preauth_token_invalid01",
            preauth_token,
        )
        .await
        .unwrap();

        similar_asserts::assert_eq!(INVALID_AUTH_TOKEN_ERROR, result);
        remove_container(&docker, "preauth_token_invalid01").await;
    }

    #[rstest]
    #[tokio::test]
    /// Create key with reuseCount=2 then create 3 nodes, verify that the key is depleted on the third node.
    async fn login_with_depleted_preauth_token(
        config: Config,
        docker: Docker,
        client: reqwest::Client,
        runtime_info: RuntimeInformation,
    ) {
        let namespace_name = "optm";
        create_namespace(namespace_name, &runtime_info, &client)
            .await
            .unwrap();
        let preauth_token = create_preauth_token(
            &client,
            &runtime_info,
            CreatePreauthTokenRequest {
                acl_tags: vec![],
                namespace: namespace_name.to_string(),
                prefix: "".to_string(),
                reuse_count: 2,
                ephemeral: false,
                expiration: "".to_string(),
            },
        )
        .await
        .unwrap();

        let good1 = create_client_with_preauth_token(
            &docker,
            &config,
            "preauth_token_depleted_good01",
            preauth_token.as_str(),
        )
        .await
        .unwrap();

        let good2 = create_client_with_preauth_token(
            &docker,
            &config,
            "preauth_token_depleted_good02",
            preauth_token.as_str(),
        )
        .await
        .unwrap();

        let bad1 = create_client_with_preauth_token(
            &docker,
            &config,
            "preauth_token_depleted_bad01",
            preauth_token.as_str(),
        )
        .await
        .unwrap();

        dbg!(&good1, &good2, &bad1);
        similar_asserts::assert_eq!("", good1);
        similar_asserts::assert_eq!("", good2);
        similar_asserts::assert_eq!(INVALID_AUTH_TOKEN_ERROR, bad1);

        remove_container(&docker, "preauth_token_depleted_good01").await;
        remove_container(&docker, "preauth_token_depleted_good02").await;
        remove_container(&docker, "preauth_token_depleted_bad01").await;
    }

    #[rstest]
    #[tokio::test]
    // Create a token expires in a second, sleep, then try to register nodes with it.
    async fn login_with_expired_preauth_token(
        config: Config,
        docker: Docker,
        client: reqwest::Client,
        runtime_info: RuntimeInformation,
    ) {
        let namespace_name = "optm";
        create_namespace(namespace_name, &runtime_info, &client)
            .await
            .unwrap();

        let preauth_token = create_preauth_token(
            &client,
            &runtime_info,
            CreatePreauthTokenRequest {
                acl_tags: vec![],
                namespace: namespace_name.to_string(),
                prefix: "".to_string(),
                reuse_count: 0,
                ephemeral: false,
                expiration: "2s".to_string(),
            },
        )
        .await
        .unwrap();

        let container_name = "preauthexp01";
        let result = create_client_with_preauth_token(
            &docker,
            &config,
            container_name,
            preauth_token.as_str(),
        )
        .await
        .unwrap();
        similar_asserts::assert_eq!(INVALID_AUTH_TOKEN_ERROR, result, "Should have failed.");
        remove_container(&docker, container_name).await;
    }

    #[rstest]
    #[tokio::test]

    async fn login_with_pat_then_switch_to_user(
        config: Config,
        docker: Docker,
        client: reqwest::Client,
        runtime_info: RuntimeInformation,
    ) {
        let namespace_name = "patswitch";
        create_namespace(namespace_name, &runtime_info, &client)
            .await
            .unwrap();

        let preauth_token = create_preauth_token(
            &client,
            &runtime_info,
            CreatePreauthTokenRequest {
                acl_tags: vec![],
                namespace: namespace_name.to_string(),
                prefix: "".to_string(),
                reuse_count: 0,
                ephemeral: false,
                expiration: "".to_string(),
            },
        )
        .await
        .unwrap();

        let container_name = "login_with_pat01";
        let result = create_client_with_preauth_token(
            &docker,
            &config,
            container_name,
            preauth_token.as_str(),
        )
        .await
        .unwrap();
        similar_asserts::assert_eq!("", result);

        let status = wait_for_state_change(&docker, container_name, RUNNING_STATE).await;
        assert!(status.is_tagged_user());

        ztclient_logout(&docker, container_name).await.unwrap();

        let user_id = 4;

        let correlation_id = ztclient_registration(&docker, container_name)
            .await
            .unwrap();
        let request = ExecuteCallbackRequest {
            correlation_id: correlation_id.as_str(),
            api_key: &runtime_info.ninja_panda_api_key,
            namespace_name,
            user_info_id: user_id,
            ninja_panda_api_url: &runtime_info.ninja_panda_api_url,
        };
        let _ = execute_callback(&client, &request).await.unwrap();

        let status = wait_for_state_change(&docker, container_name, RUNNING_STATE).await;
        let user = get_user(user_id);
        status.assert_user(&user);

        ztclient_logout(&docker, container_name).await.unwrap();

        // Register it again with the preauth token
        preauth_token_registration(
            &docker,
            container_name,
            preauth_token.as_str(),
            NGINX_NP_URL,
        )
        .await
        .unwrap();

        let status = wait_for_state_change(&docker, container_name, RUNNING_STATE).await;
        assert!(status.is_tagged_user());

        remove_container(&docker, container_name).await;
    }

    #[rstest]
    #[tokio::test]
    async fn login_with_pat_force_reauth_to_user(
        config: Config,
        docker: Docker,
        client: reqwest::Client,
        runtime_info: RuntimeInformation,
    ) {
        let namespace_name = "frcswitch";
        create_namespace(namespace_name, &runtime_info, &client)
            .await
            .unwrap();

        let preauth_token = create_preauth_token(
            &client,
            &runtime_info,
            CreatePreauthTokenRequest {
                acl_tags: vec![],
                namespace: namespace_name.to_string(),
                prefix: "".to_string(),
                reuse_count: 0,
                ephemeral: false,
                expiration: "".to_string(),
            },
        )
        .await
        .unwrap();

        let container_name = "login_with_reauth01";
        let result = create_client_with_preauth_token(
            &docker,
            &config,
            container_name,
            preauth_token.as_str(),
        )
        .await
        .unwrap();
        similar_asserts::assert_eq!("", result);

        let status = wait_for_state_change(&docker, container_name, RUNNING_STATE).await;
        assert!(status.is_tagged_user());

        let user_id = 4;

        let correlation_id = ztclient_register_forcereauth(&docker, container_name)
            .await
            .unwrap();

        let request = ExecuteCallbackRequest {
            correlation_id: correlation_id.as_str(),
            api_key: &runtime_info.ninja_panda_api_key,
            namespace_name,
            user_info_id: user_id,
            ninja_panda_api_url: &runtime_info.ninja_panda_api_url,
        };
        let _ = execute_callback(&client, &request).await.unwrap();

        let status = wait_for_state_change(&docker, container_name, RUNNING_STATE).await;
        let user = get_user(user_id);
        status.assert_user(&user);

        let _ = ztclient_logout(&docker, container_name).await;
        // ztclient_execute(&docker, container_name, vec!["ztclient", "disconnect"])
        //     .await
        //     .unwrap();

        // Now let's switch from user back to pre-auth key
        ztclient_execute(
            &docker,
            container_name,
            vec![
                "ztclient",
                "connect",
                "--url",
                NGINX_NP_URL,
                "--force-reauth",
                "--auth-token",
                preauth_token.as_str(),
            ],
        )
        .await
        .unwrap();

        let status = wait_for_state_change(&docker, container_name, RUNNING_STATE).await;
        assert!(status.is_tagged_user());

        remove_container(&docker, container_name).await;
    }

    // Login with a preauth token for a namespace that doesn't exist. (i.e. create token, then drop namespace)
    // Create an ephemeral token, create a node, logit out and then re-start it, assert that a new IP gets created
    // NinjaPanda: FailedAuthentication with authkeys, protocl_common.go:490: Add Prometheus Metric
    // Client: when key is bad, the client should give up and not try to reuse it in an infinite loop.
}
