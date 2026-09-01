// This is a generated Motoko binding.
// Please use `import service "ic:canister_id"` instead to call canisters on the IC if possible.

module {
  /// Represents an AI agent registration information in the Anda network system.
  public type Agent = {
    /// The unique identifier of the agent.
    id : Principal;
    /// Optional Trusted Execution Environment information where the agent is running.
    tee : ?TEEInfo;
    /// Timestamp when the current challenge expires.
    challenged_expiration : Nat64;
    /// Information about the agent, including name, description, and supported protocols.
    info : AgentInfo;
    /// Timestamp when the agent was registered in milliseconds since the Unix epoch.
    created_at : Nat64;
    /// Timestamp when the agent was last challenged in milliseconds since the Unix epoch.
    challenged_at : Nat64;
    /// The principal that issued the challenge to this agent.
    challenged_by : Principal;
    /// The most recent activation timestamp, updated when an agent successfully responds to a challenge
    /// after its previous challenge has expired.
    /// The agent's uptime can be calculated as: uptime = now_ms - actived_start
    actived_start : Nat64;
    /// The challenge code for the next round, used to ensure the agent is in a healthy state.
    challenge_code : Blob;
    /// Accumulated health value used to assess the agent's availability.
    /// This value increases with each successful challenge and decreases when challenges expire.
    /// 
    /// Calculation example:
    /// 1. Assuming the challenge validity period is 1 hour (3,600,000 ms)
    /// 2. When an agent is successfully challenged, health_power increases by (now_ms - challenged_at)
    /// 3. When an agent hasn't been successfully challenged for over 1 hour (e.g., challenge after 1.5 hours),
    /// health_power decreases by (now_ms - challenged_at), which would be 5,400,000
    /// and actived_start is reset to now_ms
    health_power : Nat64;
  };
  /// Contains descriptive and operational information about an AI agent.
  /// 
  /// This structure holds the metadata and configuration details that define
  /// an agent's capabilities, endpoints, and supported protocols.
  public type AgentInfo = {
    /// The dMsg.net canister where the agent profile is stored.
    handle_canister : ?Principal;
    /// Information about the agent's service provider.
    provider : ?AgentProvider;
    /// A endpoint URL for the agent. This is the URL that other agents and
    /// users will use to communicate with the agent.
    endpoint : Text;
    /// Human readable name of the agent.
    /// (e.g. "Anda ICP")
    name : Text;
    /// Communication protocols the agent supports.
    protocols : [AgentProtocol];
    /// A human-readable description of the agent. Used to assist users and
    /// other agents in understanding what the agent can do.
    /// (e.g. "Agent that helps users with recipes and cooking.")
    description : Text;
    /// Unique account identifier of the agent.
    handle : Text;
    /// A URL to an image representing the agent, such as a logo or avatar.
    /// (e.g. "https://DOMAIN/path/to/image.png")
    image : Text;
  };
  /// Information about the agent's communication protocol.
  public type AgentProtocol = {
    /// A URI for the agent protocol's endpoint.
    /// (e.g. "https://DOMAIN/.well-known/agent.json")
    endpoint : Text;
    /// The name of the agent protocol. Should be uppercase.
    /// (e.g. "A2A", "MCP", "X402")
    name : Text;
    /// The version of the agent protocol.
    /// (e.g. "v1")
    version : ?Text;
  };
  /// Information about the agent's service provider.
  public type AgentProvider = {
    /// The unique identifier of the agent provider.
    id : Principal;
    /// A URL for the agent provider's website or relevant documentation.
    url : Text;
    /// The agent provider's logo.
    logo : Text;
    /// The name of the agent provider's organization.
    name : Text;
  };
  public type ChainArgs = { #Upgrade : UpgradeArgs; #Init : InitArgs };
  /// A complete envelope containing a challenge request and its authentication.
  /// 
  /// This structure is used for secure agent registration and verification,
  /// combining the challenge data with authentication proof and optional TEE information.
  /// 
  /// The complete challenge process is as follows:
  /// 1. The challenger obtains the agent's challenge code from the Registry Canister.
  /// For first-time registration, the challenge code is [0u8; 32].
  /// 2. The challenger obtains the agent's latest information through its protocol
  /// and generates a ChallengeRequest.
  /// 3. The challenger signs the ChallengeRequest with their ICP identity and sends it to the agent.
  /// 4. Upon receiving the challenger's request, the agent confirms the information
  /// and signs it with its own ICP identity.
  /// 5. If the agent is running in a TEE environment, it also generates TEE information,
  /// including an attestation containing this challenge information.
  /// 6. The challenger sends the ChallengeEnvelope returned by the agent to the Registry Canister.
  /// 7. The Registry Canister verifies the ChallengeEnvelope and updates the agent's status.
  /// 8. Since the challenge code is updated after each successful challenge, only the first
  /// challenger for a given challenge code can succeed. Requests from other challengers
  /// for the same challenge code will be invalid.
  public type ChallengeEnvelope = {
    /// The agent's signature on the challenge request.
    authentication : SignedEnvelope;
    /// TEE information if the agent is running in a Trusted Execution Environment.
    tee : ?TEEInfo;
    /// The challenge request initiated by the challenger.
    request : ChallengeRequest;
  };
  /// Represents a challenge request initiated by a challenger to an agent.
  /// 
  /// This structure contains all the necessary information for a challenger
  /// to verify an agent's health and update its information in the registry.
  public type ChallengeRequest = {
    /// Authentication signature information from the challenger.
    /// The challenger must be registered in the Anda Registry Canister, otherwise the challenge will fail.
    authentication : ?SignedEnvelope;
    /// Latest information about the agent being challenged.
    /// When the challenge is successful, this information will be updated in the Anda Registry Canister,
    /// effectively synchronizing the agent's information through the challenge process.
    agent : AgentInfo;
    /// The challenge code that must be responded to.
    code : Blob;
    /// Creation timestamp of the challenge request in milliseconds since the Unix epoch.
    created_at : Nat64;
    /// The registry canister where the agent is registered.
    registry : Principal;
  };
  /// DelegationCompact is a compact representation of a [`Delegation`].
  /// It is used to reduce the size of the delegation when it is serialized.
  public type DelegationCompact = {
    /// The nanosecond UNIX timestamp after which the delegation is invalid,
    /// encoded as `e` in compact JSON/CBOR maps.
    e : Nat64;
    /// The delegated-to key, encoded as `p` in compact JSON/CBOR maps.
    p : Blob;
    /// Optional canister targets, encoded as `t` in compact JSON/CBOR maps.
    t : ?[Principal];
    /// Optional permissions, encoded as `perm` in compact JSON/CBOR maps.
    perm : ?DelegationPermissions;
  };
  /// The kinds of requests a [`Delegation`] permits.
  public type DelegationPermissions = {
    /// All request types are permitted.
    #all;
    /// Only query calls and `read_state` requests are permitted.
    #queries;
  };
  public type InitArgs = {
    governance_canister : ?Principal;
    name : Text;
    challenge_expires_in_ms : Nat64;
  };
  /// Represents errors that can occur during registry operations.
  /// 
  /// This enum provides specific error types with associated messages
  /// to help diagnose and handle different failure scenarios.
  public type RegistryError = {
    /// Error when an agent with the specified handle cannot be found.
    #NotFound : { handle : Text };
    /// A generic error with a descriptive message.
    #Generic : { error : Text };
    /// Error when the caller is not authenticated or lacks valid credentials.
    #Unauthorized : { error : Text };
    /// Error when attempting to register an agent with a handle that already exists.
    #AlreadyExists : { handle : Text };
    /// Error when the requested operation is not supported by the registry.
    #NotSupported : { error : Text };
    /// Error when the caller is authenticated but lacks permission for the operation.
    #Forbidden : { error : Text };
    /// Error when the request is malformed or contains invalid parameters.
    #BadRequest : { error : Text };
  };
  /// Represents the state of an Anda Registry Canister.
  /// 
  /// The Registry Canister is responsible for managing agent registrations,
  /// maintaining a list of authorized challengers, and coordinating with
  /// other registry canisters in a distributed network.
  public type RegistryState = {
    /// Maximum number of agents that can be registered in this registry.
    max_agent : Nat64;
    /// Optional principal ID of the governance canister that controls this registry.
    /// If set, certain administrative operations require approval from this canister.
    governance_canister : ?Principal;
    /// The name of this registry instance.
    name : Text;
    /// Set of principal IDs authorized to issue challenges to agents.
    /// Only principals in this set can initiate the challenge process.
    challengers : [Principal];
    /// Set of principal IDs subscribed to registry events.
    /// These subscribers receive notifications about agent registration changes.
    subscribers : [Principal];
    /// Duration in milliseconds after which a challenge expires.
    /// Agents must respond to challenges within this timeframe to maintain their active status.
    challenge_expires_in_ms : Nat64;
    /// Set of principal IDs of peer registry canisters in the network.
    /// These peers can synchronize agent information across the network.
    peers : [Principal];
    /// Set of principal IDs of name service canisters associated with this registry.
    /// These canisters handle agent name resolution and discovery.
    name_canisters : [Principal];
    /// Total number of agents currently registered in this registry.
    agents_total : Nat64;
  };
  public type Result = { #Ok; #Err : Text };
  public type Result_1 = { #Ok; #Err : RegistryError };
  public type Result_2 = { #Ok : Agent; #Err : RegistryError };
  public type Result_3 = { #Ok : RegistryState; #Err : RegistryError };
  public type Result_4 = { #Ok : [(Principal, Nat64)]; #Err : RegistryError };
  public type Result_5 = { #Ok : (Nat64, [Agent]); #Err : RegistryError };
  public type Result_6 = { #Ok : [Agent]; #Err : RegistryError };
  public type Result_7 = { #Ok : Text; #Err : Text };
  /// SignedDelegationCompact is a compact representation of a [`SignedDelegation`].
  /// It is used to reduce the size of the delegation when it is serialized.
  public type SignedDelegationCompact = {
    /// The compact delegation payload, encoded as `d`.
    d : DelegationCompact;
    /// The signature over the delegation message, encoded as `s`.
    s : Blob;
  };
  /// The authentication envelope for Internet Computer authentication.
  /// 
  /// This structure encapsulates all the necessary components for authenticating
  /// a request to an Internet Computer service. It includes the public key of the
  /// sender, the signature, the content digest, and an optional delegation chain.
  /// 
  /// The envelope can be serialized to and from various formats, including bytes,
  /// base64 strings, and HTTP headers, making it versatile for different transport
  /// mechanisms.
  public type SignedEnvelope = {
    /// The chain of delegations connecting `pubkey` to `signature`, in order.
    /// Each delegation authorizes the next entity in the chain to sign on behalf
    /// of the previous entity, forming a chain of trust from the original identity
    /// to the actual signer.
    d : ?[SignedDelegationCompact];
    /// The request content's hash digest that was signed by the sender.
    /// This is typically a SHA-256 or SHA3-256 hash of the request content.
    h : ?Blob;
    /// The public key of the self-authenticating principal this request is from.
    /// This is the head of the delegation chain (if any) and is used to derive
    /// the principal ID of the sender.
    p : Blob;
    /// A cryptographic signature authorizing the request.
    /// When delegations are involved, this is the signature from the tail of the
    /// delegation chain, not necessarily made by the owner of `pubkey`.
    s : Blob;
  };
  /// Represents information about a Trusted Execution Environment (TEE) where an agent is running.
  /// 
  /// TEEs provide hardware-based isolation and security guarantees for code execution.
  /// This structure contains identification and attestation information for a TEE instance.
  public type TEEInfo = {
    /// The principal identifier of the TEE instance.
    id : Principal;
    /// URL where additional TEE information can be retrieved.
    /// (e.g. https://DOMAIN/.well-known/tee)
    url : Text;
    /// The type of TEE technology being used.
    kind : TEEKind;
    /// Optional attestation data that proves the TEE's authenticity.
    /// This typically contains cryptographic evidence that the TEE is genuine
    /// and running the expected code.
    attestation : ?Blob;
  };
  /// Enumerates the supported types of Trusted Execution Environments.
  /// 
  /// Different TEE technologies provide varying levels of security guarantees
  /// and are supported by different hardware platforms.
  public type TEEKind = {
    /// AWS Nitro Enclaves, a TEE technology provided by Amazon Web Services.
    /// Nitro Enclaves provide isolated compute environments for sensitive workloads.
    #NITRO;
  };
  public type UpgradeArgs = {
    governance_canister : ?Principal;
    name : ?Text;
    challenge_expires_in_ms : ?Nat64;
  };
  public type Self = ?ChainArgs -> async actor {
    admin_add_challengers : shared [Principal] -> async Result;
    admin_add_name_canisters : shared [Principal] -> async Result;
    admin_add_peers : shared [Principal] -> async Result;
    admin_add_subscribers : shared [Principal] -> async Result;
    admin_remove_challengers : shared [Principal] -> async Result;
    admin_remove_name_canisters : shared [Principal] -> async Result;
    admin_remove_peers : shared [Principal] -> async Result;
    admin_remove_subscribers : shared [Principal] -> async Result;
    challenge : shared ChallengeEnvelope -> async Result_1;
    get_agent : shared query Principal -> async Result_2;
    get_agent_by_handle : shared query Text -> async Result_2;
    get_state : shared query () -> async Result_3;
    last_challenged : shared query ?Nat64 -> async Result_4;
    list : shared query (?Nat64, ?Nat64) -> async Result_5;
    list_by_health_power : shared query ?Nat64 -> async Result_6;
    register : shared ChallengeEnvelope -> async Result_1;
    validate_admin_add_challengers : shared [Principal] -> async Result_7;
    validate_admin_add_name_canisters : shared [Principal] -> async Result_7;
    validate_admin_add_peers : shared [Principal] -> async Result_7;
    validate_admin_add_subscribers : shared [Principal] -> async Result_7;
    validate_admin_remove_challengers : shared [Principal] -> async Result_7;
    validate_admin_remove_name_canisters : shared [Principal] -> async Result_7;
    validate_admin_remove_peers : shared [Principal] -> async Result_7;
    validate_admin_remove_subscribers : shared [Principal] -> async Result_7;
  }
}
