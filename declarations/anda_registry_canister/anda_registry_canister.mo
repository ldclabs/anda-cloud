// This is a generated Motoko binding.
// Please use `import service "ic:canister_id"` instead to call canisters on the IC if possible.

module {
  public type Agent = {
    id : Principal;
    tee : ?TEEInfo;
    challenged_expiration : Nat64;
    info : AgentInfo;
    created_at : Nat64;
    challenged_at : Nat64;
    challenged_by : Principal;
    actived_start : Nat64;
    challenge_code : Blob;
    health_power : Nat64;
  };
  public type AgentInfo = {
    handle_canister : ?Principal;
    provider : ?AgentProvider;
    endpoint : Text;
    name : Text;
    protocols : [AgentProtocol];
    description : Text;
    handle : Text;
    image : Text;
  };
  public type AgentProtocol = { endpoint : Text; name : Text; version : ?Text };
  public type AgentProvider = {
    id : Principal;
    url : Text;
    logo : Text;
    name : Text;
  };
  public type ChainArgs = { #Upgrade : UpgradeArgs; #Init : InitArgs };
  public type ChallengeEnvelope = {
    authentication : SignedEnvelope;
    tee : ?TEEInfo;
    request : ChallengeRequest;
  };
  public type ChallengeRequest = {
    authentication : ?SignedEnvelope;
    agent : AgentInfo;
    code : Blob;
    created_at : Nat64;
    registry : Principal;
  };
  public type DelegationCompact = { e : Nat64; p : Blob; t : ?[Principal] };
  public type InitArgs = {
    governance_canister : ?Principal;
    name : Text;
    challenge_expires_in_ms : Nat64;
  };
  public type RegistryError = {
    #NotFound : { handle : Text };
    #Generic : { error : Text };
    #Unauthorized : { error : Text };
    #AlreadyExists : { handle : Text };
    #NotSupported : { error : Text };
    #Forbidden : { error : Text };
    #BadRequest : { error : Text };
  };
  public type RegistryState = {
    max_agent : Nat64;
    governance_canister : ?Principal;
    name : Text;
    challengers : [Principal];
    subscribers : [Principal];
    challenge_expires_in_ms : Nat64;
    peers : [Principal];
    name_canisters : [Principal];
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
  public type SignedDelegationCompact = { d : DelegationCompact; s : Blob };
  public type SignedEnvelope = {
    d : ?[SignedDelegationCompact];
    h : Blob;
    p : Blob;
    s : Blob;
  };
  public type TEEInfo = {
    id : Principal;
    url : Text;
    kind : TEEKind;
    attestation : ?Blob;
  };
  public type TEEKind = { #NITRO };
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
