import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export interface AssetInfo {
  'decimals' : number,
  'transfer_fee' : bigint,
  'logo' : [] | [string],
  'name' : string,
  'payment_fee' : bigint,
  'symbol' : string,
}
export type CanisterArgs = { 'Upgrade' : UpgradeArgs } |
  { 'Init' : InitArgs };
export interface InitArgs {
  'governance_canister' : [] | [Principal],
  'name' : string,
}
export interface PayerStateInfo {
  'next_nonce' : bigint,
  'logs' : BigUint64Array | bigint[],
  'total_sent' : Array<[Principal, bigint]>,
}
export interface PaymentLogInfo {
  'id' : bigint,
  'to' : Principal,
  'fee' : string,
  'asset' : Principal,
  'value' : string,
  'scheme' : string,
  'from' : Principal,
  'nonce' : bigint,
  'timestamp' : bigint,
  'expires_at' : bigint,
}
export type Result = { 'Ok' : null } |
  { 'Err' : string };
export type Result_1 = { 'Ok' : bigint } |
  { 'Err' : string };
export type Result_2 = { 'Ok' : StateInfo } |
  { 'Err' : string };
export type Result_3 = { 'Ok' : PayerStateInfo } |
  { 'Err' : string };
export type Result_4 = { 'Ok' : Array<PaymentLogInfo> } |
  { 'Err' : string };
export type Result_5 = { 'Ok' : bigint } |
  { 'Err' : string };
export type Result_6 = { 'Ok' : string } |
  { 'Err' : string };
export interface StateInfo {
  'total_withdrawn_fees' : Array<[Principal, bigint]>,
  'supported_payments' : Array<SupportedKindCan>,
  'total_collected_fees' : Array<[Principal, bigint]>,
  'governance_canister' : [] | [Principal],
  'name' : string,
  'supported_assets' : Array<[Principal, AssetInfo]>,
  'key_name' : string,
}
export interface SupportedKindCan {
  'scheme' : string,
  'network' : string,
  'x402_version' : number,
}
export interface UpgradeArgs {
  'governance_canister' : [] | [Principal],
  'name' : [] | [string],
}
export interface _SERVICE {
  'admin_add_supported_payment' : ActorMethod<[number, string], Result>,
  'admin_collect_fees' : ActorMethod<[Principal, Principal, bigint], Result_1>,
  'admin_remove_supported_asset' : ActorMethod<[Principal], Result>,
  'admin_remove_supported_payment' : ActorMethod<[number, string], Result>,
  'admin_update_supported_asset' : ActorMethod<[Principal, bigint], Result>,
  'info' : ActorMethod<[], Result_2>,
  'my_info' : ActorMethod<[], Result_3>,
  'my_payment_logs' : ActorMethod<[number, [] | [bigint]], Result_4>,
  'next_nonce' : ActorMethod<[], Result_5>,
  'validate_admin_add_supported_payment' : ActorMethod<
    [number, string],
    Result_6
  >,
  'validate_admin_collect_fees' : ActorMethod<
    [Principal, Principal, bigint],
    Result_6
  >,
  'validate_admin_remove_supported_payment' : ActorMethod<
    [number, string],
    Result_6
  >,
  'validate_admin_update_supported_asset' : ActorMethod<
    [Principal, bigint],
    Result_6
  >,
  'validate_remove_update_supported_asset' : ActorMethod<[Principal], Result_6>,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
