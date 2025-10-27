import { Principal } from '@dfinity/principal'
import { HttpAgent, SignIdentity } from '@dfinity/agent'
import { DelegationChain, DelegationIdentity } from '@dfinity/identity'
import type { _SERVICE } from '../candid/anda_x402_canister/anda_x402_canister.did.js'
import { createActor } from '../candid/anda_x402_canister/index.js'
import {
  type _SERVICE as _TOKEN_SERVICE,
  type Allowance
} from '../candid/icrc1_ledger_canister/icrc1_ledger_canister.did.js'
import { createActor as createTokenActor } from '../candid/icrc1_ledger_canister/index.js'
import type {
  StateInfo,
  PaymentLogInfo,
  TokenInfo,
  IcpPayloadAuthorization,
  PaymentRequirementsResponse,
  X402Request,
  PaymentPayload
} from './types.js'
import {
  toDelegationIdentity,
  signCborMessage,
  deterministicEncode,
  bytesToBase64Url
} from '@ldclabs/ic-auth'

export {
  Delegation,
  DelegationChain,
  DelegationIdentity,
  Ed25519KeyIdentity,
  Ed25519PublicKey
} from '@dfinity/identity'

export class X402Canister {
  readonly canisterId: Principal
  readonly network: string
  readonly endpoint: string
  #identity: DelegationIdentity
  #agent: HttpAgent
  #actor: _SERVICE
  #tokenActors: Map<string, _TOKEN_SERVICE> = new Map()

  constructor(
    canisterId: string | Principal,
    identity: SignIdentity,
    host?: string
  ) {
    this.canisterId = toPrincipal(canisterId)
    this.network = `icp-${this.canisterId.toText()}`
    this.endpoint = `https://${this.canisterId.toText()}.icp0.io`
    this.#identity = toDelegationIdentity(identity)

    this.#agent = HttpAgent.createSync({
      host,
      identity: this.#identity,
      verifyQuerySignatures: false
    })
    if (this.#agent.isLocal()) {
      this.#agent.fetchRootKey()
    }
    this.#actor = createActor(canisterId, { agent: this.#agent })
  }

  setIdentity(identity: SignIdentity) {
    this.#identity = toDelegationIdentity(identity)
    this.#agent.replaceIdentity(this.#identity)
  }

  async nextNonce(): Promise<number> {
    const res = await this.#actor.next_nonce()
    return Number(resultOk(res))
  }

  async signPayload(data: IcpPayloadAuthorization): Promise<string> {
    const sig = await signCborMessage(this.#identity, data)
    delete sig.h
    return bytesToBase64Url(deterministicEncode(sig))
  }

  async buildX402Request(
    res: PaymentRequirementsResponse,
    asset: string
  ): Promise<X402Request> {
    for (const req of res.accepts) {
      if (req.network == this.network && req.asset === asset) {
        const [info, nonce] = await Promise.all([
          this.getInfo(),
          this.nextNonce()
        ])

        if (!info.supportedAssets[asset]) {
          throw new Error(`Asset ${asset} not supported by x402 facilitator`)
        }

        const now = Math.floor(Date.now() / 1000)
        const authorization: IcpPayloadAuthorization = {
          scheme: req.scheme,
          asset: req.asset,
          to: req.payTo,
          value: req.maxAmountRequired,
          expiresAt: (now + req.maxTimeoutSeconds) * 1000,
          nonce: nonce
        }
        const signature = await this.signPayload(authorization)

        return {
          paymentPayload: {
            x402Version: res.x402Version,
            scheme: req.scheme,
            network: req.network,
            payload: {
              signature,
              authorization
            }
          },
          paymentRequirements: req
        }
      }
    }

    throw new Error(`Asset ${asset} not accepted`)
  }

  async getInfo(): Promise<StateInfo> {
    const _res = await this.#actor.info()
    const res = resultOk(_res)
    return {
      name: res.name,
      supportedPayments: res.supported_payments.map((sp) => ({
        scheme: 'Exact' in sp.scheme ? 'exact' : 'upto',
        network: sp.network,
        x402Version: 'V1' in sp.x402_version ? 1 : 0
      })),
      supportedAssets: Object.fromEntries(
        res.supported_assets.map(([principal, info]) => [
          principal.toText(),
          {
            symbol: info.symbol,
            decimals: info.decimals,
            transferFee: info.transfer_fee,
            paymentFee: info.payment_fee
          }
        ])
      ),
      totalWithdrawnFees: Object.fromEntries(
        res.total_withdrawn_fees.map(([principal, amount]) => [
          principal.toText(),
          amount
        ])
      ),
      totalCollectedFees: Object.fromEntries(
        res.total_collected_fees.map(([principal, amount]) => [
          principal.toText(),
          amount
        ])
      ),
      governanceCanister: res.governance_canister?.[0]?.toText()
    }
  }

  async listMyPaymentLogs(
    take: number,
    prev?: bigint
  ): Promise<PaymentLogInfo[]> {
    const _res = await this.#actor.my_payment_logs(take, prev ? [prev] : [])
    const res = resultOk(_res)
    return res.map((log) => ({
      id: log.id,
      to: log.to.toText(),
      fee: log.fee,
      asset: log.asset.toText(),
      value: log.value,
      scheme: 'Exact' in log.scheme ? 'exact' : 'upto',
      from: log.from.toText(),
      nonce: log.nonce,
      timestamp: log.timestamp,
      expiresAt: log.expires_at
    }))
  }

  #getTokenActor(tokenCanisterId: string | Principal): _TOKEN_SERVICE {
    const canisterIdText = toText(tokenCanisterId)
    let actor = this.#tokenActors.get(canisterIdText)
    if (!actor) {
      actor = createTokenActor(tokenCanisterId, { agent: this.#agent })
      this.#tokenActors.set(canisterIdText, actor)
    }
    return actor
  }

  async getTokenInfo(tokenCanisterId: string | Principal): Promise<TokenInfo> {
    const metadata = await this.#getTokenActor(tokenCanisterId).icrc1_metadata()

    const token: TokenInfo = {
      name: '',
      symbol: '',
      decimals: 0,
      fee: 0n,
      logo: '',
      canisterId: toPrincipal(tokenCanisterId)
    }

    for (const [key, value] of metadata) {
      switch (key) {
        case 'icrc1:name':
          token.name = (value as { 'Text': string }).Text
          continue
        case 'icrc1:symbol':
          token.symbol = (value as { 'Text': string }).Text
          continue
        case 'icrc1:decimals':
          const decimals = (value as { 'Nat': bigint }).Nat
          token.decimals = Number(decimals)
          continue
        case 'icrc1:fee':
          token.fee = (value as { 'Nat': bigint }).Nat
          continue
        case 'icrc1:logo':
          token.logo = (value as { 'Text': string }).Text
          continue
      }
    }

    return token
  }

  async getBalanceOf(
    tokenCanisterId: string | Principal,
    owner?: string | Principal
  ): Promise<bigint> {
    return this.#getTokenActor(tokenCanisterId).icrc1_balance_of({
      owner: toPrincipal(owner || this.#identity.getPrincipal()),
      subaccount: []
    })
  }

  async allowance(
    tokenCanisterId: string | Principal,
    spender?: string | Principal
  ): Promise<Allowance> {
    return this.#getTokenActor(tokenCanisterId).icrc2_allowance({
      account: { owner: this.#identity.getPrincipal(), subaccount: [] },
      spender: {
        owner: toPrincipal(spender || this.canisterId),
        subaccount: []
      }
    })
  }

  async approve(
    tokenCanisterId: string | Principal,
    amount: bigint,
    spender?: string | Principal
  ): Promise<bigint> {
    const res = await this.#getTokenActor(tokenCanisterId).icrc2_approve({
      fee: [],
      memo: [],
      from_subaccount: [],
      created_at_time: [],
      amount: amount,
      expected_allowance: [],
      expires_at: [],
      spender: {
        owner: toPrincipal(spender || this.canisterId),
        subaccount: []
      }
    })

    return resultOk(res)
  }

  async ensureAllowance(
    tokenCanisterId: string | Principal,
    amount: bigint,
    spender?: string | Principal
  ): Promise<void> {
    const allowance = await this.allowance(tokenCanisterId, spender)
    const expires_at = allowance.expires_at[0] || 0n
    if (
      allowance.allowance < amount ||
      (expires_at > 0 && expires_at < BigInt((Date.now() + 60000) * 1_000_000))
    ) {
      await this.approve(tokenCanisterId, amount, spender)
    }
  }

  async transfer(
    tokenCanisterId: string | Principal,
    to: string | Principal,
    amount: bigint
  ): Promise<bigint> {
    const principal = toPrincipal(to)
    const res = await this.#getTokenActor(tokenCanisterId).icrc1_transfer({
      from_subaccount: [],
      to: { owner: principal, subaccount: [] },
      amount,
      fee: [],
      memo: [],
      created_at_time: []
    })

    return resultOk(res)
  }
}

function toText(principal: Principal | string): string {
  return typeof principal === 'string' ? principal : principal.toText()
}

function toPrincipal(principal: Principal | string): Principal {
  return typeof principal === 'string'
    ? Principal.fromText(principal)
    : principal
}

interface Ok<T> {
  Ok: T
}

interface Err<T> {
  Err: T
}

type Result<T, E> = Ok<T> | Err<E>

function resultOk<T, E>(res: Result<T, E>): T {
  if ('Err' in res) {
    throw res.Err
  }

  return res.Ok
}
