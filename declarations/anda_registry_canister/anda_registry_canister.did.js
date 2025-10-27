export const idlFactory = ({ IDL }) => {
  const UpgradeArgs = IDL.Record({
    'governance_canister' : IDL.Opt(IDL.Principal),
    'name' : IDL.Opt(IDL.Text),
    'challenge_expires_in_ms' : IDL.Opt(IDL.Nat64),
  });
  const InitArgs = IDL.Record({
    'governance_canister' : IDL.Opt(IDL.Principal),
    'name' : IDL.Text,
    'challenge_expires_in_ms' : IDL.Nat64,
  });
  const ChainArgs = IDL.Variant({ 'Upgrade' : UpgradeArgs, 'Init' : InitArgs });
  const Result = IDL.Variant({ 'Ok' : IDL.Null, 'Err' : IDL.Text });
  const DelegationCompact = IDL.Record({
    'e' : IDL.Nat64,
    'p' : IDL.Vec(IDL.Nat8),
    't' : IDL.Opt(IDL.Vec(IDL.Principal)),
  });
  const SignedDelegationCompact = IDL.Record({
    'd' : DelegationCompact,
    's' : IDL.Vec(IDL.Nat8),
  });
  const SignedEnvelope = IDL.Record({
    'd' : IDL.Opt(IDL.Vec(SignedDelegationCompact)),
    'h' : IDL.Opt(IDL.Vec(IDL.Nat8)),
    'p' : IDL.Vec(IDL.Nat8),
    's' : IDL.Vec(IDL.Nat8),
  });
  const TEEKind = IDL.Variant({ 'NITRO' : IDL.Null });
  const TEEInfo = IDL.Record({
    'id' : IDL.Principal,
    'url' : IDL.Text,
    'kind' : TEEKind,
    'attestation' : IDL.Opt(IDL.Vec(IDL.Nat8)),
  });
  const AgentProvider = IDL.Record({
    'id' : IDL.Principal,
    'url' : IDL.Text,
    'logo' : IDL.Text,
    'name' : IDL.Text,
  });
  const AgentProtocol = IDL.Record({
    'endpoint' : IDL.Text,
    'name' : IDL.Text,
    'version' : IDL.Opt(IDL.Text),
  });
  const AgentInfo = IDL.Record({
    'handle_canister' : IDL.Opt(IDL.Principal),
    'provider' : IDL.Opt(AgentProvider),
    'endpoint' : IDL.Text,
    'name' : IDL.Text,
    'protocols' : IDL.Vec(AgentProtocol),
    'description' : IDL.Text,
    'handle' : IDL.Text,
    'image' : IDL.Text,
  });
  const ChallengeRequest = IDL.Record({
    'authentication' : IDL.Opt(SignedEnvelope),
    'agent' : AgentInfo,
    'code' : IDL.Vec(IDL.Nat8),
    'created_at' : IDL.Nat64,
    'registry' : IDL.Principal,
  });
  const ChallengeEnvelope = IDL.Record({
    'authentication' : SignedEnvelope,
    'tee' : IDL.Opt(TEEInfo),
    'request' : ChallengeRequest,
  });
  const RegistryError = IDL.Variant({
    'NotFound' : IDL.Record({ 'handle' : IDL.Text }),
    'Generic' : IDL.Record({ 'error' : IDL.Text }),
    'Unauthorized' : IDL.Record({ 'error' : IDL.Text }),
    'AlreadyExists' : IDL.Record({ 'handle' : IDL.Text }),
    'NotSupported' : IDL.Record({ 'error' : IDL.Text }),
    'Forbidden' : IDL.Record({ 'error' : IDL.Text }),
    'BadRequest' : IDL.Record({ 'error' : IDL.Text }),
  });
  const Result_1 = IDL.Variant({ 'Ok' : IDL.Null, 'Err' : RegistryError });
  const Agent = IDL.Record({
    'id' : IDL.Principal,
    'tee' : IDL.Opt(TEEInfo),
    'challenged_expiration' : IDL.Nat64,
    'info' : AgentInfo,
    'created_at' : IDL.Nat64,
    'challenged_at' : IDL.Nat64,
    'challenged_by' : IDL.Principal,
    'actived_start' : IDL.Nat64,
    'challenge_code' : IDL.Vec(IDL.Nat8),
    'health_power' : IDL.Nat64,
  });
  const Result_2 = IDL.Variant({ 'Ok' : Agent, 'Err' : RegistryError });
  const RegistryState = IDL.Record({
    'max_agent' : IDL.Nat64,
    'governance_canister' : IDL.Opt(IDL.Principal),
    'name' : IDL.Text,
    'challengers' : IDL.Vec(IDL.Principal),
    'subscribers' : IDL.Vec(IDL.Principal),
    'challenge_expires_in_ms' : IDL.Nat64,
    'peers' : IDL.Vec(IDL.Principal),
    'name_canisters' : IDL.Vec(IDL.Principal),
    'agents_total' : IDL.Nat64,
  });
  const Result_3 = IDL.Variant({ 'Ok' : RegistryState, 'Err' : RegistryError });
  const Result_4 = IDL.Variant({
    'Ok' : IDL.Vec(IDL.Tuple(IDL.Principal, IDL.Nat64)),
    'Err' : RegistryError,
  });
  const Result_5 = IDL.Variant({
    'Ok' : IDL.Tuple(IDL.Nat64, IDL.Vec(Agent)),
    'Err' : RegistryError,
  });
  const Result_6 = IDL.Variant({
    'Ok' : IDL.Vec(Agent),
    'Err' : RegistryError,
  });
  const Result_7 = IDL.Variant({ 'Ok' : IDL.Text, 'Err' : IDL.Text });
  return IDL.Service({
    'admin_add_challengers' : IDL.Func([IDL.Vec(IDL.Principal)], [Result], []),
    'admin_add_name_canisters' : IDL.Func(
        [IDL.Vec(IDL.Principal)],
        [Result],
        [],
      ),
    'admin_add_peers' : IDL.Func([IDL.Vec(IDL.Principal)], [Result], []),
    'admin_add_subscribers' : IDL.Func([IDL.Vec(IDL.Principal)], [Result], []),
    'admin_remove_challengers' : IDL.Func(
        [IDL.Vec(IDL.Principal)],
        [Result],
        [],
      ),
    'admin_remove_name_canisters' : IDL.Func(
        [IDL.Vec(IDL.Principal)],
        [Result],
        [],
      ),
    'admin_remove_peers' : IDL.Func([IDL.Vec(IDL.Principal)], [Result], []),
    'admin_remove_subscribers' : IDL.Func(
        [IDL.Vec(IDL.Principal)],
        [Result],
        [],
      ),
    'challenge' : IDL.Func([ChallengeEnvelope], [Result_1], []),
    'get_agent' : IDL.Func([IDL.Principal], [Result_2], ['query']),
    'get_agent_by_handle' : IDL.Func([IDL.Text], [Result_2], ['query']),
    'get_state' : IDL.Func([], [Result_3], ['query']),
    'last_challenged' : IDL.Func([IDL.Opt(IDL.Nat64)], [Result_4], ['query']),
    'list' : IDL.Func(
        [IDL.Opt(IDL.Nat64), IDL.Opt(IDL.Nat64)],
        [Result_5],
        ['query'],
      ),
    'list_by_health_power' : IDL.Func(
        [IDL.Opt(IDL.Nat64)],
        [Result_6],
        ['query'],
      ),
    'register' : IDL.Func([ChallengeEnvelope], [Result_1], []),
    'validate_admin_add_challengers' : IDL.Func(
        [IDL.Vec(IDL.Principal)],
        [Result_7],
        [],
      ),
    'validate_admin_add_name_canisters' : IDL.Func(
        [IDL.Vec(IDL.Principal)],
        [Result_7],
        [],
      ),
    'validate_admin_add_peers' : IDL.Func(
        [IDL.Vec(IDL.Principal)],
        [Result_7],
        [],
      ),
    'validate_admin_add_subscribers' : IDL.Func(
        [IDL.Vec(IDL.Principal)],
        [Result_7],
        [],
      ),
    'validate_admin_remove_challengers' : IDL.Func(
        [IDL.Vec(IDL.Principal)],
        [Result_7],
        [],
      ),
    'validate_admin_remove_name_canisters' : IDL.Func(
        [IDL.Vec(IDL.Principal)],
        [Result_7],
        [],
      ),
    'validate_admin_remove_peers' : IDL.Func(
        [IDL.Vec(IDL.Principal)],
        [Result_7],
        [],
      ),
    'validate_admin_remove_subscribers' : IDL.Func(
        [IDL.Vec(IDL.Principal)],
        [Result_7],
        [],
      ),
  });
};
export const init = ({ IDL }) => {
  const UpgradeArgs = IDL.Record({
    'governance_canister' : IDL.Opt(IDL.Principal),
    'name' : IDL.Opt(IDL.Text),
    'challenge_expires_in_ms' : IDL.Opt(IDL.Nat64),
  });
  const InitArgs = IDL.Record({
    'governance_canister' : IDL.Opt(IDL.Principal),
    'name' : IDL.Text,
    'challenge_expires_in_ms' : IDL.Nat64,
  });
  const ChainArgs = IDL.Variant({ 'Upgrade' : UpgradeArgs, 'Init' : InitArgs });
  return [IDL.Opt(ChainArgs)];
};
