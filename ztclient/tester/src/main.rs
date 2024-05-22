use anyhow::{Context, Result};
use bollard::{network::CreateNetworkOptions, Docker};
use clap::{Args, Parser, Subcommand};
use dotenv::dotenv;

use std::{fs::File, io::Write};
use ztclient_common::{
    execute_callback, get_running_json,
    ninjapanda::{create_namespace, start_ninjapanda},
    ztclient::{preauth_token_registration, start_ztclientd, ztclient_registration},
    Config, ExecuteCallbackRequest, RuntimeInformation,
};

#[derive(Parser, Debug)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Creates an environment
    CreateEnvironment(CreateEnvironmentArgs),
    /// Launch and register a specified number of clients by invoking the register/callback endpoint.
    RegisterClients(RegisterClientsArgs),
    /// Launch and register a specified number of clients with a pre-auth key
    LaunchClients(LaunchClientsArgs),
}

#[derive(Debug, Default, Args)]
pub struct CreateEnvironmentArgs {
    #[arg(
        long,
        help = "The number of clients to start with this test suite",
        default_value = "4"
    )]
    num_clients: u32,

    #[arg(
        long,
        help = "The port at which to expose the Ninja Panda API",
        default_value = "15000"
    )]
    ninja_panda_port: u32,

    #[arg(
        long,
        help = "Sleep time in seconds between starting the backends (DB, Kafka) and starting NinjaPanda",
        default_value = "5"
    )]
    sleep_time: u64,

    #[arg(
        long,
        help = "The port at which to expose the Postgres SQL database",
        default_value = "15100"
    )]
    database_port: u32,

    #[arg(
        long,
        help = "The hostname prefix that will be given as hostname to the machines",
        default_value = "ztclienthost"
    )]
    hostname_prefix: String,
}

impl CreateEnvironmentArgs {
    fn validate(self) -> Self {
        self
    }
    async fn execute(&self, config: &Config) -> Result<()> {
        let opts = self;
        let mut running_json_file = File::create("running.json").unwrap();

        // Initialize Docker client
        let docker = Docker::connect_with_unix_defaults()?;
        log::info!("Obtained connection to Docker daemon");

        log::info!("Starting primary NinjaPanda and will get API key");
        let api_key = start_ninjapanda(&docker, config.ninja_panda_container_name.as_str()).await?;

        let ninja_panda_api_url: String = format!("http://localhost:{}", opts.ninja_panda_port);

        let runtime_info = RuntimeInformation {
            ninja_panda_api_key: api_key.clone(),
            ninja_panda_api_url: ninja_panda_api_url.clone(),
        };
        let ri_bytes = serde_json::to_vec(&runtime_info).unwrap();
        running_json_file.write_all(&ri_bytes).unwrap();
        Ok(())
    }
}

#[derive(Debug, Default, Args)]
pub struct LaunchClientsArgs {
    #[arg(
        short = 'c',
        long,
        help = "The number of clients to start with this test suite",
        default_value = "1"
    )]
    num_clients: u32,

    #[arg(
        short = 'o',
        long,
        help = "An offset for starting client numbering at something other than prefix_001 but rather prefix_offset",
        default_value = "0"
    )]
    offset: u32,

    #[arg(
        short = 't',
        long,
        required = true,
        help = "Pre-auth token used to login the clients"
    )]
    pre_auth_token: String,

    #[arg(short, long, required = true, help = "URL to reach Ninja Panda")]
    url: String,

    #[arg(
        short = 'm',
        long,
        required = true,
        help = "Machine hostname prefix for the new client hostnames"
    )]
    hostname_prefix: String,
}

impl LaunchClientsArgs {
    async fn execute(&self, config: &Config) -> Result<()> {
        // Initialize Docker client
        let docker = Docker::connect_with_unix_defaults()?;

        // We assume that this network-create will succeed.  We do not delete it
        // because other containers may be using it, nor will we (yet) check that
        // the failure was because it was already existing.
        let _ = docker
            .create_network(CreateNetworkOptions {
                name: config.docker_network_name.as_str(),
                check_duplicate: true,
                ..Default::default()
            })
            .await;

        log::info!("Re-created docker network for all the containers");

        for x in 1..self.num_clients + 1 {
            let container_name = format!("{}{:0>3}", self.hostname_prefix, x + self.offset);

            start_ztclientd(&docker, config, &container_name)
                .await
                .unwrap();
            preauth_token_registration(&docker, &container_name, &self.pre_auth_token, &self.url)
                .await
                .unwrap();
        }
        Ok(())
    }
}

#[derive(Debug, Default, Args)]
pub struct RegisterClientsArgs {
    #[arg(
        short = 'c',
        long,
        help = "The number of clients to start with this test suite",
        default_value = "1"
    )]
    num_clients: u32,

    #[arg(
        short = 'o',
        long,
        help = "An offset for starting client numbering at something other than prefix_001 but rather prefix_offset",
        default_value = "0"
    )]
    offset: u32,

    #[arg(
        short = 'm',
        long,
        required = true,
        help = "Machine hostname prefix for the new client hostnames"
    )]
    hostname_prefix: String,

    #[arg(
        short = 'n',
        long,
        help = "Namespace to use for registering the machine",
        default_value = "optm"
    )]
    namespace_name: String,
}

impl RegisterClientsArgs {
    async fn execute(&self, config: &Config) -> Result<()> {
        // Initialize Docker client
        let docker = Docker::connect_with_unix_defaults()?;

        // We assume that this network-create will succeed.  We do not delete it
        // because other containers may be using it, nor will we (yet) check that
        // the failure was because it was already existing.
        let _ = docker
            .create_network(CreateNetworkOptions {
                name: config.docker_network_name.as_str(),
                check_duplicate: true,
                ..Default::default()
            })
            .await;

        log::info!("Re-created docker network for all the containers");

        let runtime_info = get_running_json()
            .with_context(|| "Unable to open runtime information")
            .unwrap();

        let client = reqwest::Client::new();
        // We really do not care if the namespace exists already.
        create_namespace(&self.namespace_name, &runtime_info, &client)
            .await
            .unwrap();

        for x in 1..self.num_clients + 1 {
            let container_name = format!("{}{:0>3}", self.hostname_prefix, x + self.offset);

            start_ztclientd(&docker, config, &container_name)
                .await
                .unwrap();
            let correlation_id = ztclient_registration(&docker, container_name.as_str()).await?;
            let request = ExecuteCallbackRequest {
                correlation_id: correlation_id.as_str(),
                api_key: runtime_info.ninja_panda_api_key.as_str(),
                namespace_name: self.namespace_name.as_str(),
                user_info_id: (x as usize % 9),
                ninja_panda_api_url: runtime_info.ninja_panda_api_url.as_str(),
            };
            execute_callback(&client, &request).await?;
        }
        Ok(())
    }
}
#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    // Parse the .env file
    dotenv().ok();

    let config = match envy::from_env::<Config>() {
        Ok(config) => config,
        Err(error) => panic!("{:#?}", error),
    };

    // Parse command line arguments using Clap and fix dependent variables
    let opts: Cli = Cli::parse();

    match opts.command {
        Command::CreateEnvironment(x) => {
            x.validate().execute(&config).await?;
        }
        Command::RegisterClients(x) => {
            x.execute(&config).await?;
        }
        Command::LaunchClients(x) => {
            x.execute(&config).await?;
        }
    };
    Ok(())
}
