use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterCallbackRequest {
    pub namespace: String,
    pub user_info: UserInfo,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserInfo {
    pub user_info_id: String,
    pub email: String,
    pub display_name: String,
    pub created_at: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateNamespaceRequest {
    pub name: String,
    pub default_machine_key_ttl: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetMachinesResponse {
    pub machines: Vec<Machine>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MachineUpdateMessage {
    pub machine: Machine,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteCallbackResponse {
    pub machine: Machine,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Machine {
    pub machine_id: String,
    pub machine_key: String,
    pub node_key: String,
    pub session_key: String,
    pub ip_addresses: Vec<String>,
    pub name: String,
    pub namespace: Namespace,
    pub last_seen: String,
    pub last_successful_update: Option<String>,
    pub expiry: String,
    pub pre_auth_key: Option<Value>,
    pub created_at: String,
    pub register_method: Option<String>,
    pub forced_tags: Option<Vec<Value>>,
    pub invalid_tags: Option<Vec<Value>>,
    pub valid_tags: Option<Vec<Value>>,
    pub given_name: String,
    pub online: bool,
    pub os: String,
    pub os_version: String,
    pub package: Option<String>,
    pub hostname: String,
    pub client_version: String,
    pub machine_location: MachineLocation,
    pub preferred_relay: String,
    pub relay_latency: RelayLatency,
    pub user_info: Option<UserInfo>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelayLatency {}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Namespace {
    pub name: String,
    pub created_at: String,
    pub default_machine_key_ttl: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MachineLocation {
    pub city: String,
    pub region: String,
    pub region_code: Option<String>,
    pub country: String,
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAclPolicyRequest {
    pub acl_policy: AclPolicy,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAclPolicyRequest {
    pub acl_policies: Vec<AclPolicy>,
}
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AclPolicy {
    #[serde(rename = "aclpolicy_id")]
    pub aclpolicy_id: String,
    pub order: String,
    #[serde(rename = "groups")]
    pub groups: Vec<Group>,
    pub acls: Vec<Acl>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Group {
    pub key: String,
    pub values: Vec<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Acl {
    pub order: i64,
    pub action: String,
    pub port: String,
    pub protocol: String,
    pub sources: Vec<String>,
    pub destinations: Vec<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePreauthTokenRequest {
    pub namespace: String,
    pub prefix: String,
    pub reuse_count: u64,
    pub ephemeral: bool,
    pub expiration: String,
    pub acl_tags: Vec<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreauthTokenResponse {
    pub pre_auth_key: PreauthToken,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreauthToken {
    pub pre_auth_key_id: String,
    pub namespace: String,
    pub key: String,
    pub prefix: String,
    pub reuse_count: String,
    pub ephemeral: bool,
    pub expiration: Option<String>,
    pub created_at: String,
    pub revoked_at: Option<String>,
    pub status: String,
    pub acl: Option<Vec<String>>,
}

pub mod status {
    use crate::users::UserInfo;

    use super::*;

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct StatusResult {
        #[serde(rename = "Version")]
        pub version: String,
        #[serde(rename = "BackendState")]
        pub backend_state: String,
        #[serde(rename = "AuthURL")]
        pub auth_url: String,
        #[serde(rename = "ZTMeshIPs")]
        pub ztmesh_ips: Option<Vec<String>>,
        #[serde(rename = "Self")]
        pub self_field: StatusNode,
        #[serde(rename = "Health")]
        pub health: Value,
        #[serde(rename = "MagicDNSSuffix")]
        pub magic_dnssuffix: String,
        #[serde(rename = "CurrentZTnet")]
        pub current_ztnet: Option<CurrentZtnet>,
        #[serde(rename = "CertDomains")]
        pub cert_domains: Value,
        #[serde(rename = "Peer")]
        pub peer: Option<HashMap<String, StatusNode>>,
        #[serde(rename = "User")]
        pub user: Option<HashMap<String, StatusUserInfo>>,
    }

    impl StatusResult {
        pub fn is_tagged_user(&self) -> bool {
            if let Some(user) = &self.user {
                let obj = user.iter().nth(0).unwrap();
                obj.1.login_name == "tagged-devices"
            } else {
                false
            }
        }
        pub fn assert_user(&self, user_info: &UserInfo) -> bool {
            if let Some(user) = &self.user {
                let obj = user.iter().nth(0).unwrap();
                obj.1.login_name == user_info.email
                    && obj.1.first_name == user_info.first_name
                    && obj.1.last_name == user_info.last_name
            } else {
                false
            }
        }
    }
    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct CurrentZtnet {
        #[serde(rename = "Name")]
        pub name: String,
        #[serde(rename = "MagicDNSSuffix")]
        pub magic_dnssuffix: String,
        #[serde(rename = "MagicDNSEnabled")]
        pub magic_dnsenabled: bool,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct StatusNode {
        #[serde(rename = "ID")]
        pub id: String,
        #[serde(rename = "PublicKey")]
        pub public_key: String,
        #[serde(rename = "HostName")]
        pub host_name: String,
        #[serde(rename = "DNSName")]
        pub dnsname: String,
        #[serde(rename = "OS")]
        pub os: String,
        #[serde(rename = "UserID")]
        pub user_id: i64,
        #[serde(rename = "ZTMeshIPs")]
        pub ztmesh_ips: Option<Vec<String>>,
        #[serde(rename = "Addrs")]
        pub addrs: Value,
        #[serde(rename = "CurAddr")]
        pub cur_addr: String,
        #[serde(rename = "Relay")]
        pub relay: String,
        #[serde(rename = "RxBytes")]
        pub rx_bytes: i64,
        #[serde(rename = "TxBytes")]
        pub tx_bytes: i64,
        #[serde(rename = "Created")]
        pub created: String,
        #[serde(rename = "LastWrite")]
        pub last_write: String,
        #[serde(rename = "LastSeen")]
        pub last_seen: String,
        #[serde(rename = "LastHandshake")]
        pub last_handshake: String,
        #[serde(rename = "Online")]
        pub online: bool,
        #[serde(rename = "KeepAlive")]
        pub keep_alive: bool,
        #[serde(rename = "ExitNode")]
        pub exit_node: bool,
        #[serde(rename = "ExitNodeOption")]
        pub exit_node_option: bool,
        #[serde(rename = "Active")]
        pub active: bool,
        #[serde(rename = "PeerAPIURL")]
        pub peer_apiurl: Option<Vec<String>>,
        #[serde(rename = "InNetworkMap")]
        pub in_network_map: bool,
        #[serde(rename = "InMagicSock")]
        pub in_magic_sock: bool,
        #[serde(rename = "InEngine")]
        pub in_engine: bool,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct User {
        #[serde(rename = "1")]
        pub n1: StatusUserInfo,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct StatusUserInfo {
        #[serde(rename = "ID")]
        pub id: i64,
        #[serde(rename = "LoginName")]
        pub login_name: String,
        #[serde(rename = "FirstName")]
        pub first_name: String,
        #[serde(rename = "LastName")]
        pub last_name: String,
        #[serde(rename = "DisplayName")]
        pub display_name: String,
        #[serde(rename = "ProfilePicURL")]
        pub profile_pic_url: String,
        #[serde(rename = "Roles")]
        pub roles: Vec<Value>,
    }

    impl StatusUserInfo {
        pub fn assert_eq(&self, user_info: &UserInfo) {
            assert_eq!(
                self.first_name, user_info.first_name,
                "User first name mismatch"
            );
            assert_eq!(
                self.last_name, user_info.last_name,
                "User last name mismatch"
            );
        }
    }
}

pub mod ztcon {
    use super::*;

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "PascalCase")]
    pub struct ConResult {
        pub elapsed_time: Option<u64>,
        pub name: String,
        pub address: String,
    }
}

pub mod ztn {
    use std::collections::HashMap;

    use serde::{Deserialize, Serialize};
    use serde_json::Value;

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ZtnMessage {
        #[serde(rename = "Version")]
        pub version: String,
        #[serde(rename = "ErrMessage")]
        pub err_message: Value,
        #[serde(rename = "LoginFinished")]
        pub login_finished: Value,
        #[serde(rename = "State")]
        pub state: Option<i64>,
        #[serde(rename = "Prefs")]
        pub prefs: Option<Prefs>,
        #[serde(rename = "NetMap")]
        pub net_map: Option<NetMap>,
        #[serde(rename = "Engine")]
        pub engine: Option<Value>,
        #[serde(rename = "Latencies")]
        pub latencies: Option<Value>,
        #[serde(rename = "BrowseToURL")]
        pub browse_to_url: Value,
        #[serde(rename = "BackendLogID")]
        pub backend_log_id: Value,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Prefs {
        #[serde(rename = "ControlURL")]
        pub control_url: String,
        #[serde(rename = "RouteAll")]
        pub route_all: bool,
        #[serde(rename = "AllowSingleHosts")]
        pub allow_single_hosts: bool,
        #[serde(rename = "ExitNodeID")]
        pub exit_node_id: String,
        #[serde(rename = "ExitNodeIP")]
        pub exit_node_ip: String,
        #[serde(rename = "ExitNodeAllowLANAccess")]
        pub exit_node_allow_lanaccess: bool,
        #[serde(rename = "CorpDNS")]
        pub corp_dns: bool,
        #[serde(rename = "RunSSH")]
        pub run_ssh: bool,
        #[serde(rename = "WantRunning")]
        pub want_running: bool,
        #[serde(rename = "LoggedOut")]
        pub logged_out: bool,
        #[serde(rename = "ClientOnly")]
        pub client_only: bool,
        #[serde(rename = "AdvertiseTags")]
        pub advertise_tags: Value,
        #[serde(rename = "Hostname")]
        pub hostname: String,
        #[serde(rename = "NotepadURLs")]
        pub notepad_urls: bool,
        #[serde(rename = "AdvertiseRoutes")]
        pub advertise_routes: Value,
        #[serde(rename = "NoSNAT")]
        pub no_snat: bool,
        #[serde(rename = "NetfilterMode")]
        pub netfilter_mode: i64,
        #[serde(rename = "AutoConnect")]
        pub auto_connect: bool,
        #[serde(rename = "Config")]
        pub config: Config,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Config {
        #[serde(rename = "PrivateMachineKey")]
        pub private_machine_key: String,
        #[serde(rename = "PrivateNodeKey")]
        pub private_node_key: String,
        #[serde(rename = "OldPrivateNodeKey")]
        pub old_private_node_key: String,
        #[serde(rename = "Provider")]
        pub provider: String,
        #[serde(rename = "LoginName")]
        pub login_name: String,
        #[serde(rename = "UserProfile")]
        pub user_profile: UserProfile,
        #[serde(rename = "NetworkLockKey")]
        pub network_lock_key: String,
        #[serde(rename = "NodeID")]
        pub node_id: String,
        #[serde(rename = "KeySignAuthority")]
        pub key_sign_authority: KeySignAuthority,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct UserProfile {
        #[serde(rename = "ID")]
        pub id: i64,
        #[serde(rename = "LoginName")]
        pub login_name: String,
        #[serde(rename = "DisplayName")]
        pub display_name: String,
        #[serde(rename = "FirstName")]
        pub first_name: String,
        #[serde(rename = "LastName")]
        pub last_name: String,
        #[serde(rename = "ProfilePicURL")]
        pub profile_pic_url: String,
        #[serde(rename = "Roles")]
        pub roles: Vec<Value>,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct KeySignAuthority {
        #[serde(rename = "KeyServerUrl")]
        pub key_server_url: String,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct NetMap {
        #[serde(rename = "SelfNode")]
        pub self_node: SelfNode,
        #[serde(rename = "NodeKey")]
        pub node_key: String,
        #[serde(rename = "PrivateKey")]
        pub private_key: String,
        #[serde(rename = "Expiry")]
        pub expiry: String,
        #[serde(rename = "Name")]
        pub name: String,
        #[serde(rename = "Addresses")]
        pub addresses: Vec<String>,
        #[serde(rename = "MachineStatus")]
        pub machine_status: String,
        #[serde(rename = "MachineKey")]
        pub machine_key: String,
        #[serde(rename = "Peers")]
        pub peers: Option<Vec<SelfNode>>,
        #[serde(rename = "DNS")]
        pub dns: Dns,
        #[serde(rename = "Hostinfo")]
        pub hostinfo: Option<Hostinfo>,
        #[serde(rename = "PacketFilter")]
        pub packet_filter: Option<Vec<PacketFilter>>,
        #[serde(rename = "SSHPolicy")]
        pub sshpolicy: Value,
        #[serde(rename = "CollectServices")]
        pub collect_services: bool,
        #[serde(rename = "RELAYMap")]
        pub relaymap: Relaymap,
        #[serde(rename = "Debug")]
        pub debug: Option<Debug>,
        #[serde(rename = "ControlHealth")]
        pub control_health: Value,
        #[serde(rename = "TKAEnabled")]
        pub tkaenabled: bool,
        #[serde(rename = "TKAHead")]
        pub tkahead: String,
        #[serde(rename = "User")]
        pub user: i64,
        #[serde(rename = "Domain")]
        pub domain: String,
        #[serde(rename = "DomainAuditLogID")]
        pub domain_audit_log_id: String,
        #[serde(rename = "UserProfiles")]
        pub user_profiles: HashMap<String, UserProfile>,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct SelfNode {
        #[serde(rename = "ID")]
        pub id: i64,
        #[serde(rename = "StableID")]
        pub stable_id: String,
        #[serde(rename = "Name")]
        pub name: String,
        #[serde(rename = "User")]
        pub user: i64,
        #[serde(rename = "Key")]
        pub key: String,
        #[serde(rename = "KeyExpiry")]
        pub key_expiry: String,
        #[serde(rename = "Machine")]
        pub machine: String,
        #[serde(rename = "SessionKey")]
        pub session_key: String,
        #[serde(rename = "Addresses")]
        pub addresses: Vec<String>,
        #[serde(rename = "AllowedIPs")]
        pub allowed_ips: Vec<String>,
        #[serde(rename = "RELAY")]
        pub relay: String,
        #[serde(rename = "Hostinfo")]
        #[serde(default)]
        pub hostinfo: Hostinfo,
        #[serde(rename = "Created")]
        pub created: String,
        #[serde(rename = "PrimaryRoutes")]
        pub primary_routes: Option<Vec<String>>,
        #[serde(rename = "LastSeen")]
        pub last_seen: Option<String>,
        #[serde(rename = "Online")]
        pub online: bool,
        #[serde(rename = "KeepAlive")]
        pub keep_alive: bool,
        #[serde(rename = "MachineAuthorized")]
        pub machine_authorized: bool,
        #[serde(rename = "Capabilities")]
        pub capabilities: Vec<String>,
        #[serde(rename = "ComputedName")]
        pub computed_name: String,
        #[serde(rename = "ComputedNameWithHost")]
        pub computed_name_with_host: String,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    #[serde(default)]
    pub struct Hostinfo {
        #[serde(rename = "ZTMVersion")]
        #[serde(default)]
        pub ztmversion: String,
        #[serde(rename = "BackendLogID")]
        #[serde(default)]
        pub backend_log_id: String,
        #[serde(rename = "OS")]
        #[serde(default)]
        pub os: String,
        #[serde(rename = "OSVersion")]
        #[serde(default)]
        pub osversion: String,
        #[serde(rename = "Container")]
        #[serde(default)]
        pub container: bool,
        #[serde(rename = "Distro")]
        #[serde(default)]
        pub distro: String,
        #[serde(rename = "DistroVersion")]
        #[serde(default)]
        pub distro_version: String,
        #[serde(rename = "Desktop")]
        #[serde(default)]
        pub desktop: bool,
        #[serde(rename = "Hostname")]
        #[serde(default)]
        pub hostname: String,
        #[serde(rename = "GoArch")]
        #[serde(default)]
        pub go_arch: String,
        #[serde(rename = "GoVersion")]
        #[serde(default)]
        pub go_version: String,
        #[serde(rename = "Services")]
        #[serde(default)]
        pub services: Vec<Service>,
        #[serde(rename = "Userspace")]
        #[serde(default)]
        pub userspace: bool,
        #[serde(rename = "UserspaceRouter")]
        #[serde(default)]
        pub userspace_router: bool,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Service {
        #[serde(rename = "Proto")]
        pub proto: String,
        #[serde(rename = "Port")]
        pub port: i64,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct NetInfo {
        #[serde(rename = "MappingVariesByDestIP")]
        pub mapping_varies_by_dest_ip: Value,
        #[serde(rename = "HairPinning")]
        pub hair_pinning: bool,
        #[serde(rename = "WorkingIPv6")]
        pub working_ipv6: bool,
        #[serde(rename = "OSHasIPv6")]
        pub oshas_ipv6: bool,
        #[serde(rename = "WorkingUDP")]
        pub working_udp: bool,
        #[serde(rename = "WorkingICMPv4")]
        pub working_icmpv4: bool,
        #[serde(rename = "UPnP")]
        pub upn_p: bool,
        #[serde(rename = "PMP")]
        pub pmp: bool,
        #[serde(rename = "PCP")]
        pub pcp: bool,
        #[serde(rename = "PreferredRELAY")]
        pub preferred_relay: i64,
        #[serde(rename = "RELAYLatency")]
        pub relaylatency: Relaylatency,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Relaylatency {
        #[serde(rename = "2-v4")]
        pub n2_v4: f64,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Dns {
        #[serde(rename = "Resolvers")]
        pub resolvers: Vec<Resolver>,
        #[serde(rename = "Routes")]
        pub routes: Routes,
        #[serde(rename = "Domains")]
        pub domains: Vec<String>,
        #[serde(rename = "Proxied")]
        pub proxied: bool,
        #[serde(rename = "Nameservers")]
        pub nameservers: Vec<String>,
        #[serde(rename = "ExitNodeFilteredSet")]
        pub exit_node_filtered_set: Value,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct PacketFilter {
        #[serde(rename = "IPProto")]
        pub ipproto: String,
        #[serde(rename = "Srcs")]
        pub srcs: Vec<String>,
        #[serde(rename = "Dsts")]
        pub dsts: Vec<Dst>,
        #[serde(rename = "Caps")]
        pub caps: Vec<Value>,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Dst {
        #[serde(rename = "Net")]
        pub net: String,
        #[serde(rename = "Ports")]
        pub ports: Ports,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Ports {
        #[serde(rename = "First")]
        pub first: i64,
        #[serde(rename = "Last")]
        pub last: i64,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Resolver {
        #[serde(rename = "Addr")]
        pub addr: String,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Routes {}

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Relaymap {
        #[serde(rename = "Regions")]
        pub regions: HashMap<String, Region>,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Regions {}

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Region {
        #[serde(rename = "RegionID")]
        pub region_id: i64,
        #[serde(rename = "RegionCode")]
        pub region_code: String,
        #[serde(rename = "RegionName")]
        pub region_name: String,
        #[serde(rename = "Nodes")]
        pub nodes: Vec<Node>,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Node {
        #[serde(rename = "Name")]
        pub name: String,
        #[serde(rename = "RegionID")]
        pub region_id: i64,
        #[serde(rename = "HostName")]
        pub host_name: String,
        #[serde(rename = "IPv4")]
        pub ipv4: String,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Debug {
        #[serde(rename = "DisableZtmLog")]
        pub disable_ztm_log: Option<bool>,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct UserProfiles {}
}

pub mod routes {
    use serde::Deserialize;
    use serde::Serialize;

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct CreateRouteRequest {
        pub routes: Vec<Route>,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct CreateRouteResponse {
        pub routes: Vec<Route>,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Route {
        pub route_id: Option<String>,
        pub machine_id: Option<String>,
        pub prefix: String,
        pub enabled: bool,
        pub advertised: bool,
        pub is_primary: bool,
    }
}
