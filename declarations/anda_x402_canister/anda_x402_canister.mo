// This is a generated Motoko binding.
// Please use `import service "ic:canister_id"` instead to call canisters on the IC if possible.

module {
  public type AssetInfo = {
    decimals : Nat8;
    transfer_fee : Nat;
    logo : ?Text;
    name : Text;
    payment_fee : Nat;
    symbol : Text;
  };
  public type CanisterArgs = { #Upgrade : UpgradeArgs; #Init : InitArgs };
  public type InitArgs = { governance_canister : ?Principal; name : Text };
  public type PaymentLogInfo = {
    id : Nat64;
    to : Principal;
    fee : Text;
    asset : Principal;
    value : Text;
    scheme : Scheme;
    from : Principal;
    nonce : Nat64;
    timestamp : Nat64;
    expires_at : Nat64;
  };
  public type Result = { #Ok; #Err : Text };
  public type Result_1 = { #Ok : Nat; #Err : Text };
  public type Result_2 = { #Ok : State; #Err : Text };
  public type Result_3 = { #Ok : [PaymentLogInfo]; #Err : Text };
  public type Result_4 = { #Ok : Nat64; #Err : Text };
  public type Result_5 = { #Ok : Text; #Err : Text };
  public type Scheme = { #Exact; #Upto };
  public type State = {
    total_withdrawn_fees : [(Principal, Nat)];
    supported_payments : [SupportedPaymentKind];
    total_collected_fees : [(Principal, Nat)];
    governance_canister : ?Principal;
    name : Text;
    supported_assets : [(Principal, AssetInfo)];
  };
  public type SupportedPaymentKind = {
    scheme : Scheme;
    network : Text;
    x402_version : X402Version;
  };
  public type UpgradeArgs = { governance_canister : ?Principal; name : ?Text };
  public type X402Version = { #V1 };
  public type Self = ?CanisterArgs -> async actor {
    admin_add_supported_payment : shared (X402Version, Scheme) -> async Result;
    admin_collect_fees : shared (Principal, Principal, Nat) -> async Result_1;
    admin_remove_supported_asset : shared Principal -> async Result;
    admin_remove_supported_payment : shared (
        X402Version,
        Scheme,
      ) -> async Result;
    admin_update_supported_asset : shared (Principal, Nat) -> async Result;
    info : shared query () -> async Result_2;
    my_payment_logs : shared query (Nat32, ?Nat64) -> async Result_3;
    next_nonce : shared query () -> async Result_4;
    validate_admin_add_supported_payment : shared (
        X402Version,
        Scheme,
      ) -> async Result_5;
    validate_admin_collect_fees : shared (
        Principal,
        Principal,
        Nat,
      ) -> async Result_5;
    validate_admin_remove_supported_payment : shared (
        X402Version,
        Scheme,
      ) -> async Result_5;
    validate_admin_update_supported_asset : shared (
        Principal,
        Nat,
      ) -> async Result_5;
    validate_remove_update_supported_asset : shared Principal -> async Result_5;
  }
}
