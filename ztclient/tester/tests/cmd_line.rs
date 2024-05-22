use rstest::{fixture, rstest};

mod cmd_line_tests {
    use super::*;
    use bollard::Docker;

    use ztclient_common::{
        containers::remove_container,
        get_running_json,
        ztclient::{start_ztclientd, ztclient_execute},
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
    async fn command_line_tests(docker: Docker, config: Config) {
        let container_name = "command_line_test";
        start_ztclientd(&docker, &config, container_name)
            .await
            .unwrap();

        struct TestCli {
            commands: Vec<&'static str>,
            want: &'static str,
        }

        let tests = vec![
            TestCli {
                commands: vec!["ztclient"],
                want: r#"USAGE
  ztclient [flags] <subcommand> [command flags]

For help on subcommands, add --help after: "ztclient connect --help".

SUBCOMMANDS
  connect     Connect to ZTMesh
  disconnect  Disconnect from ZTMesh
  configure   Configure preferences
  login       Authenticate to ZTMesh
  logout      Logout from ZTMesh
  show-net    Display local network information
  show-ip     Display ZTClient IP addresses
  status      Display status of ZTClient and connections
  zt-ping     Test the ping and route to another ZTClient
  zt-con      Test the connection to a port on another ZTClient
  show-ver    Display ZTClient version
  legal       Display legal information

FLAGS
  --socket string
    	path to ztclientd's unix socket (default /var/run/ztclient/ztclientd.sock)
"#,
            },
            TestCli {
                commands: vec!["ztclient", "connect", "--help"],
                want: r#"USAGE
  connect [flags]

"ztclient connect" establishes a connection from this device to the ZTMesh 
  (if authentication is needed, it will be initiated)

FLAGS
  --auth-token string
    	node authorization token; if it begins with "file:", then it's a path to a file containing the auth token
  --auto-connect, --auto-connect=false
    	attempt to connect to ZTMesh when the client process starts (default true)
  --client-only, --client-only=false
    	block all incoming connections (default false)
  --force-reauth, --force-reauth=false
    	force reauthentication (default false)
  --hostname string
    	override the hostname to use instead of the one set in the Operating System
  --internet-gateway string
    	ZTMesh Internet Gateway (IP or base name) for internet traffic, or empty string to not use an internet gateway
  --internet-gateway-allow-local-lan, --internet-gateway-allow-local-lan=false
    	Allow direct access to the local network when routing traffic via an internet gateway (default false)
  --netfilter-mode string
    	netfilter mode (one of on, nodivert, off) (default on)
  --reset, --reset=false
    	reset unspecified settings to their default values (default false)
  --snat-subnet-routes, --snat-subnet-routes=false
    	source NAT traffic to local routes advertised with --advertise-routes (default true)
  --url string
    	base URL of ZTMesh server
  --use-dns, --use-dns=false
    	Use ZTMesh DNS settings (default true)
  --use-gateway, --use-gateway=false
    	Use ZTM Gateways when they are available (default true)
  --user string
    	Run ztclientd from a different user
"#,
            },
            TestCli {
                commands: vec!["ztclient", "disconnect", "--help"],
                want: r#"USAGE
  disconnect
"#,
            },
            TestCli {
                commands: vec!["ztclient", "configure", "--help"],
                want: r#"USAGE
  configure [flags]

"ztclient configure" sets preferences to their specified values.

Only settings explicitly mentioned will be set. There are no default values.
Note this difference when using these with "ztclient connect"


FLAGS
  --auto-connect, --auto-connect=false
    	attempt to connect to ZTMesh when the client process starts
  --client-only, --client-only=false
    	block all incoming connections
  --hostname string
    	override the hostname to use instead of the one set in the Operating System
  --use-dns, --use-dns=false
    	Use ZTMesh DNS settings
  --use-gateway, --use-gateway=false
    	Use ZTM Gateways when they are available
"#,
            },
            TestCli {
                commands: vec!["ztclient", "login", "--help"],
                want: r#"USAGE
  login [flags]

"ztclient login" logs this device into ZTMesh.

FLAGS
  --auth-token string
    	node authorization token; if it begins with "file:", then it's a path to a file containing the auth token
  --url string
    	base URL of ZTMesh server
"#,
            },
            TestCli {
                commands: vec!["ztclient", "logout", "--help"],
                want: r#"USAGE
  logout

"ztclient logout" disconnects from ZTMesh and logs out the current user.
"#,
            },
            TestCli {
                commands: vec!["ztclient", "show-net", "--help"],
                want: r#"USAGE
  show-net

FLAGS
  --every duration
    	update the show-net output every "duration" seconds (default 0s)
  --format string
    	output format; empty (for human-readable), "json" or "json-line"
  --verbose, --verbose=false
    	verbose logs (default false)
"#,
            },
            TestCli {
                commands: vec!["ztclient", "show-ip", "--help"],
                want: r#"USAGE
  show-ip [-1] [-4] [-6] [peer hostname or ip address]

Show ZTClient IP addresses for the current device.

FLAGS
  --1, --1=false
    	only print one IP address (default false)
  --4, --4=false
    	only print IPv4 address (default false)
  --6, --6=false
    	only print IPv6 address (default false)
"#,
            },
            TestCli {
                commands: vec!["ztclient", "status", "--help"],
                want: r#"USAGE
  status [flags]

FLAGS
  --active, --active=false
    	filter output to only peers with active sessions (default false)
  --json, --json=false
    	output in JSON format (default false)
  --peers, --peers=false
    	show status of peers (default true)
  --self, --self=false
    	show status of local device (default true)
"#,
            },
            TestCli {
                commands: vec!["ztclient", "zt-ping", "--help"],
                want: r#"USAGE
  zt-ping <hostname-or-IP>

<hostname-or-IP> must be valid device IP or hostname on ZTMesh.

FLAGS
  --c int
    	max number of pings to send (default 10)
  --icmp, --icmp=false
    	do a ICMP-level ping (through WireGuard, but not the local host OS stack) (default false)
  --timeout duration
    	timeout before giving up on a ping (default 5s)
"#,
            },
            TestCli {
                commands: vec!["ztclient", "zt-con", "--help"],
                want: r#"USAGE
  zt-con [-u] <hostname-or-IP> <port>

FLAGS
  --u, --u=false
    	when specified, use UDP protocol instead of the default TCP (default false)
"#,
            },
            TestCli {
                commands: vec!["ztclient", "show-ver", "--help"],
                want: r#"USAGE
  show-ver [flags]

FLAGS
  --daemon, --daemon=false
    	also print local node's daemon version (default false)
"#,
            },
            TestCli {
                commands: vec!["ztclient", "legal", "--help"],
                want: r#"USAGE
  legal

Display privacy, terms and licensing urls
"#,
            },
            TestCli {
                commands: vec!["ztclient", "legal"],
                want: r#"
ZTClient wouldn't be possible without the contributions of thousands of open
source developers. To see the open source packages included in ZTClient and
their respective license information, visit:

    https://cyberight.com/company/open-source-credits
"#,
            },
        ];

        let mut results = Vec::new();
        for test in tests.iter() {
            let answer = ztclient_execute(&docker, container_name, test.commands.clone())
                .await
                .unwrap();
            let got: &str = answer.first().unwrap().as_str();
            let result = std::panic::catch_unwind(|| {
                similar_asserts::assert_eq!(test.want, got);
            });
            results.push(result);
        }
        let error_count = results.iter().filter(|x| Result::is_err(*x)).count();
        remove_container(&docker, container_name).await;
        assert_eq!(0, error_count, "No errors were expected");
    }
}
