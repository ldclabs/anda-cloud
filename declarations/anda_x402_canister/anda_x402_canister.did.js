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
  const X402Version = IDL.Variant({ 'V1' : IDL.Null });
  const Scheme = IDL.Variant({ 'Exact' : IDL.Null, 'Upto' : IDL.Null });
  const Result = IDL.Variant({ 'Ok' : IDL.Null, 'Err' : IDL.Text });
  const Result_1 = IDL.Variant({ 'Ok' : IDL.Nat, 'Err' : IDL.Text });
  const SupportedPaymentKind = IDL.Record({
    'scheme' : Scheme,
    'network' : IDL.Text,
    'x402_version' : X402Version,
  });
  const AssetInfo = IDL.Record({
    'decimals' : IDL.Nat8,
    'transfer_fee' : IDL.Nat,
    'payment_fee' : IDL.Nat,
    'symbol' : IDL.Text,
  });
  const State = IDL.Record({
    'total_withdrawn_fees' : IDL.Vec(IDL.Tuple(IDL.Principal, IDL.Nat)),
    'supported_payments' : IDL.Vec(SupportedPaymentKind),
    'total_collected_fees' : IDL.Vec(IDL.Tuple(IDL.Principal, IDL.Nat)),
    'governance_canister' : IDL.Opt(IDL.Principal),
    'name' : IDL.Text,
    'supported_assets' : IDL.Vec(IDL.Tuple(IDL.Principal, AssetInfo)),
  });
  const Result_2 = IDL.Variant({ 'Ok' : State, 'Err' : IDL.Text });
  const PaymentLogInfo = IDL.Record({
    'id' : IDL.Nat64,
    'to' : IDL.Principal,
    'fee' : IDL.Text,
    'asset' : IDL.Principal,
    'value' : IDL.Text,
    'scheme' : Scheme,
    'from' : IDL.Principal,
    'nonce' : IDL.Nat64,
    'timestamp' : IDL.Nat64,
    'expires_at' : IDL.Nat64,
  });
  const Result_3 = IDL.Variant({
    'Ok' : IDL.Vec(PaymentLogInfo),
    'Err' : IDL.Text,
  });
  const Result_4 = IDL.Variant({ 'Ok' : IDL.Nat64, 'Err' : IDL.Text });
  const Result_5 = IDL.Variant({ 'Ok' : IDL.Text, 'Err' : IDL.Text });
  return IDL.Service({
    'admin_add_supported_payment' : IDL.Func(
        [X402Version, Scheme],
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
        [X402Version, Scheme],
        [Result],
        [],
      ),
    'admin_update_supported_asset' : IDL.Func(
        [IDL.Principal, IDL.Nat],
        [Result],
        [],
      ),
    'info' : IDL.Func([], [Result_2], ['query']),
    'my_payment_logs' : IDL.Func(
        [IDL.Nat32, IDL.Opt(IDL.Nat64)],
        [Result_3],
        ['query'],
      ),
    'next_nonce' : IDL.Func([], [Result_4], ['query']),
    'validate_admin_add_supported_payment' : IDL.Func(
        [X402Version, Scheme],
        [Result_5],
        [],
      ),
    'validate_admin_collect_fees' : IDL.Func(
        [IDL.Principal, IDL.Principal, IDL.Nat],
        [Result_5],
        [],
      ),
    'validate_admin_remove_supported_payment' : IDL.Func(
        [X402Version, Scheme],
        [Result_5],
        [],
      ),
    'validate_admin_update_supported_asset' : IDL.Func(
        [IDL.Principal, IDL.Nat],
        [Result_5],
        [],
      ),
    'validate_remove_update_supported_asset' : IDL.Func(
        [IDL.Principal],
        [Result_5],
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
