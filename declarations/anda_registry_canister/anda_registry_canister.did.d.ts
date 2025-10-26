import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export interface Agent {
  'id' : Principal,
  'tee' : [] | [TEEInfo],
  'challenged_expiration' : bigint,
  'info' : AgentInfo,
  'created_at' : bigint,
  'challenged_at' : bigint,
  'challenged_by' : Principal,
  'actived_start' : bigint,
  'challenge_code' : Uint8Array | number[],
  'health_power' : bigint,
}
export interface AgentInfo {
  'handle_canister' : [] | [Principal],
  'provider' : [] | [AgentProvider],
  'endpoint' : string,
  'name' : string,
  'protocols' : Array<AgentProtocol>,
  'description' : string,
  'handle' : string,
  'image' : string,
}
export interface AgentProtocol {
  'endpoint' : string,
  'name' : string,
  'version' : [] | [string],
}
export interface AgentProvider {
  'id' : Principal,
  'url' : string,
  'logo' : string,
  'name' : string,
}
export type ChainArgs = { 'Upgrade' : UpgradeArgs } |
  { 'Init' : InitArgs };
export interface ChallengeEnvelope {
  'authentication' : SignedEnvelope,
  'tee' : [] | [TEEInfo],
  'request' : ChallengeRequest,
}
export interface ChallengeRequest {
  'authentication' : [] | [SignedEnvelope],
  'agent' : AgentInfo,
  'code' : Uint8Array | number[],
  'created_at' : bigint,
  'registry' : Principal,
}
export interface DelegationCompact {
  'e' : bigint,
  'p' : Uint8Array | number[],
  't' : [] | [Array<Principal>],
}
export interface InitArgs {
  'governance_canister' : [] | [Principal],
  'name' : string,
  'challenge_expires_in_ms' : bigint,
}
export type RegistryError = { 'NotFound' : { 'handle' : string } } |
  { 'Generic' : { 'error' : string } } |
  { 'Unauthorized' : { 'error' : string } } |
  { 'AlreadyExists' : { 'handle' : string } } |
  { 'NotSupported' : { 'error' : string } } |
  { 'Forbidden' : { 'error' : string } } |
  { 'BadRequest' : { 'error' : string } };
export interface RegistryState {
  'max_agent' : bigint,
  'governance_canister' : [] | [Principal],
  'name' : string,
  'challengers' : Array<Principal>,
  'subscribers' : Array<Principal>,
  'challenge_expires_in_ms' : bigint,
  'peers' : Array<Principal>,
  'name_canisters' : Array<Principal>,
  'agents_total' : bigint,
}
export type Result = { 'Ok' : null } |
  { 'Err' : string };
export type Result_1 = { 'Ok' : null } |
  { 'Err' : RegistryError };
export type Result_2 = { 'Ok' : Agent } |
  { 'Err' : RegistryError };
export type Result_3 = { 'Ok' : RegistryState } |
  { 'Err' : RegistryError };
export type Result_4 = { 'Ok' : Array<[Principal, bigint]> } |
  { 'Err' : RegistryError };
export type Result_5 = { 'Ok' : [bigint, Array<Agent>] } |
  { 'Err' : RegistryError };
export type Result_6 = { 'Ok' : Array<Agent> } |
  { 'Err' : RegistryError };
export type Result_7 = { 'Ok' : string } |
  { 'Err' : string };
export interface SignedDelegationCompact {
  'd' : DelegationCompact,
  's' : Uint8Array | number[],
}
export interface SignedEnvelope {
  'd' : [] | [Array<SignedDelegationCompact>],
  'h' : [] | [Uint8Array | number[]],
  'p' : Uint8Array | number[],
  's' : Uint8Array | number[],
}
export interface TEEInfo {
  'id' : Principal,
  'url' : string,
  'kind' : TEEKind,
  'attestation' : [] | [Uint8Array | number[]],
}
export type TEEKind = { 'NITRO' : null };
export interface UpgradeArgs {
  'governance_canister' : [] | [Principal],
  'name' : [] | [string],
  'challenge_expires_in_ms' : [] | [bigint],
}
export interface _SERVICE {
  'admin_add_challengers' : ActorMethod<[Array<Principal>], Result>,
  'admin_add_name_canisters' : ActorMethod<[Array<Principal>], Result>,
  'admin_add_peers' : ActorMethod<[Array<Principal>], Result>,
  'admin_add_subscribers' : ActorMethod<[Array<Principal>], Result>,
  'admin_remove_challengers' : ActorMethod<[Array<Principal>], Result>,
  'admin_remove_name_canisters' : ActorMethod<[Array<Principal>], Result>,
  'admin_remove_peers' : ActorMethod<[Array<Principal>], Result>,
  'admin_remove_subscribers' : ActorMethod<[Array<Principal>], Result>,
  'challenge' : ActorMethod<[ChallengeEnvelope], Result_1>,
  'get_agent' : ActorMethod<[Principal], Result_2>,
  'get_agent_by_handle' : ActorMethod<[string], Result_2>,
  'get_state' : ActorMethod<[], Result_3>,
  'last_challenged' : ActorMethod<[[] | [bigint]], Result_4>,
  'list' : ActorMethod<[[] | [bigint], [] | [bigint]], Result_5>,
  'list_by_health_power' : ActorMethod<[[] | [bigint]], Result_6>,
  'register' : ActorMethod<[ChallengeEnvelope], Result_1>,
  'validate_admin_add_challengers' : ActorMethod<[Array<Principal>], Result_7>,
  'validate_admin_add_name_canisters' : ActorMethod<
    [Array<Principal>],
    Result_7
  >,
  'validate_admin_add_peers' : ActorMethod<[Array<Principal>], Result_7>,
  'validate_admin_add_subscribers' : ActorMethod<[Array<Principal>], Result_7>,
  'validate_admin_remove_challengers' : ActorMethod<
    [Array<Principal>],
    Result_7
  >,
  'validate_admin_remove_name_canisters' : ActorMethod<
    [Array<Principal>],
    Result_7
  >,
  'validate_admin_remove_peers' : ActorMethod<[Array<Principal>], Result_7>,
  'validate_admin_remove_subscribers' : ActorMethod<
    [Array<Principal>],
    Result_7
  >,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
