export const idlFactory = ({ IDL }) => {
  const UpgradeArgs = IDL.Record({
    'governance_canister' : IDL.Opt(IDL.Principal),
    'name' : IDL.Opt(IDL.Text),
  });
  const InitArgs = IDL.Record({
    'governance_canister' : IDL.Opt(IDL.Principal),
    'name' : IDL.Text,
  });
  const CanisterArgs = IDL.Variant({
    'Upgrade' : UpgradeArgs,
    'Init' : InitArgs,
  });
  const Result = IDL.Variant({ 'Ok' : IDL.Null, 'Err' : IDL.Text });
  const Result_1 = IDL.Variant({ 'Ok' : IDL.Nat, 'Err' : IDL.Text });
  const SupportedKindCan = IDL.Record({
    'scheme' : IDL.Text,
    'network' : IDL.Text,
    'x402_version' : IDL.Nat8,
  });
  const AssetInfo = IDL.Record({
    'decimals' : IDL.Nat8,
    'transfer_fee' : IDL.Nat,
    'logo' : IDL.Opt(IDL.Text),
    'name' : IDL.Text,
    'payment_fee' : IDL.Nat,
    'symbol' : IDL.Text,
  });
  const StateInfo = IDL.Record({
    'total_withdrawn_fees' : IDL.Vec(IDL.Tuple(IDL.Principal, IDL.Nat)),
    'supported_payments' : IDL.Vec(SupportedKindCan),
    'total_collected_fees' : IDL.Vec(IDL.Tuple(IDL.Principal, IDL.Nat)),
    'governance_canister' : IDL.Opt(IDL.Principal),
    'name' : IDL.Text,
    'supported_assets' : IDL.Vec(IDL.Tuple(IDL.Principal, AssetInfo)),
    'key_name' : IDL.Text,
  });
  const Result_2 = IDL.Variant({ 'Ok' : StateInfo, 'Err' : IDL.Text });
  const PayerStateInfo = IDL.Record({
    'next_nonce' : IDL.Nat64,
    'logs' : IDL.Vec(IDL.Nat64),
    'total_sent' : IDL.Vec(IDL.Tuple(IDL.Principal, IDL.Nat)),
  });
  const Result_3 = IDL.Variant({ 'Ok' : PayerStateInfo, 'Err' : IDL.Text });
  const PaymentLogInfo = IDL.Record({
    'id' : IDL.Nat64,
    'to' : IDL.Principal,
    'fee' : IDL.Text,
    'asset' : IDL.Principal,
    'value' : IDL.Text,
    'scheme' : IDL.Text,
    'from' : IDL.Principal,
    'nonce' : IDL.Nat64,
    'timestamp' : IDL.Nat64,
    'expires_at' : IDL.Nat64,
  });
  const Result_4 = IDL.Variant({
    'Ok' : IDL.Vec(PaymentLogInfo),
    'Err' : IDL.Text,
  });
  const Result_5 = IDL.Variant({ 'Ok' : IDL.Nat64, 'Err' : IDL.Text });
  const Result_6 = IDL.Variant({ 'Ok' : IDL.Text, 'Err' : IDL.Text });
  return IDL.Service({
    'admin_add_supported_payment' : IDL.Func(
        [IDL.Nat8, IDL.Text],
        [Result],
        [],
      ),
    'admin_collect_fees' : IDL.Func(
        [IDL.Principal, IDL.Principal, IDL.Nat],
        [Result_1],
        [],
      ),
    'admin_remove_supported_asset' : IDL.Func([IDL.Principal], [Result], []),
    'admin_remove_supported_payment' : IDL.Func(
        [IDL.Nat8, IDL.Text],
        [Result],
        [],
      ),
    'admin_update_supported_asset' : IDL.Func(
        [IDL.Principal, IDL.Nat],
        [Result],
        [],
      ),
    'info' : IDL.Func([], [Result_2], ['query']),
    'my_info' : IDL.Func([], [Result_3], ['query']),
    'my_payment_logs' : IDL.Func(
        [IDL.Nat32, IDL.Opt(IDL.Nat64)],
        [Result_4],
        ['query'],
      ),
    'next_nonce' : IDL.Func([], [Result_5], ['query']),
    'validate_admin_add_supported_payment' : IDL.Func(
        [IDL.Nat8, IDL.Text],
        [Result_6],
        [],
      ),
    'validate_admin_collect_fees' : IDL.Func(
        [IDL.Principal, IDL.Principal, IDL.Nat],
        [Result_6],
        [],
      ),
    'validate_admin_remove_supported_payment' : IDL.Func(
        [IDL.Nat8, IDL.Text],
        [Result_6],
        [],
      ),
    'validate_admin_update_supported_asset' : IDL.Func(
        [IDL.Principal, IDL.Nat],
        [Result_6],
        [],
      ),
    'validate_remove_update_supported_asset' : IDL.Func(
        [IDL.Principal],
        [Result_6],
        [],
      ),
  });
};
export const init = ({ IDL }) => {
  const UpgradeArgs = IDL.Record({
    'governance_canister' : IDL.Opt(IDL.Principal),
    'name' : IDL.Opt(IDL.Text),
  });
  const InitArgs = IDL.Record({
    'governance_canister' : IDL.Opt(IDL.Principal),
    'name' : IDL.Text,
  });
  const CanisterArgs = IDL.Variant({
    'Upgrade' : UpgradeArgs,
    'Init' : InitArgs,
  });
  return [IDL.Opt(CanisterArgs)];
};
