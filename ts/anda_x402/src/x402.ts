import { Actor, HttpAgent, type SignIdentity } from '@icp-sdk/core/agent'
import { DelegationIdentity } from '@icp-sdk/core/identity'
import { Principal } from '@icp-sdk/core/principal'
import {
  bytesToBase64Url,
  deterministicEncode,
  signMessage,
  toDelegationIdentity
} from '@ldclabs/ic-auth'
import {
  idlFactory as x402IdlFactory,
  type _SERVICE
} from '../candid/anda_x402_canister/anda_x402_canister.did.js'
import {
  idlFactory as tokenIdlFactory,
  type _SERVICE as _TOKEN_SERVICE,
  type Allowance
} from '../candid/icrc1_ledger_canister/icrc1_ledger_canister.did.js'
import {
  ASSET_TRANSFER_METHOD_ICRC2,
  EXTRA_ASSET_TRANSFER_METHOD,
  EXTRA_PAYMENT_FLOW,
  NETWORK_ICP,
  SUPPORTED_PAYMENT_FLOWS,
  X402_VERSION,
  assetTransferMethodOf,
  isSupportedRequirement,
  paymentFlowOf
} from './types.js'
import type {
  IcpPayload,
  IcpPayloadAuthorization,
  PaymentLogInfo,
  PaymentRequired,
  PaymentRequirements,
  StateInfo,
  SupportedResponse,
  TokenInfo,
  X402Request
} from './types.js'

export {
  Delegation,
  DelegationChain,
  DelegationIdentity,
  Ed25519KeyIdentity,
  Ed25519PublicKey
} from '@icp-sdk/core/identity'

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
    this.network = NETWORK_ICP
    this.endpoint = `https://${this.canisterId.toText()}.icp0.io`
    this.#identity = toDelegationIdentity(identity)

    this.#agent = HttpAgent.createSync({
      host,
      identity: this.#identity,
      verifyQuerySignatures: false,
      // A local replica has its own root key that can only be discovered at
      // runtime. Against mainnet we must keep the root key pinned in
      // `@icp-sdk/core`, otherwise a malicious host could hand us its own key
      // and forge the certificates that authenticate payment calls.
      // The agent fetches it lazily and awaits it before every call.
      shouldFetchRootKey: isLocalHost(host)
    })
    this.#actor = Actor.createActor<_SERVICE>(x402IdlFactory, {
      agent: this.#agent,
      canisterId: this.canisterId
    })
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
    const sig = await signMessage(this.#identity, data)
    delete sig.h // remove 'h' field as it's not needed for verification
    return bytesToBase64Url(deterministicEncode(sig))
  }

  async buildX402RequestFrom(
    res: PaymentRequired,
    asset: string
  ): Promise<X402Request<IcpPayload>> {
    if (res.x402Version !== X402_VERSION) {
      throw new Error(
        `x402 version ${res.x402Version} not supported, expected ${X402_VERSION}`
      )
    }

    // Entries whose reserved `extra` keys this SDK does not recognize are
    // skipped rather than rejected: another entry may still be payable.
    const candidates = res.accepts.filter(
      (req) =>
        req.network === this.network &&
        req.asset === asset &&
        isSupportedRequirement(req)
    )
    if (candidates.length === 0) {
      throw new Error(`Asset ${asset} not accepted`)
    }

    // Prefer post-handler settlement when the resource offers both, so the
    // funds only move once the resource has actually delivered.
    const req =
      candidates.find((r) => paymentFlowOf(r) === 'authorization') ??
      candidates[0]!

    const request = await this.buildX402Request(req, res.x402Version)
    request.paymentPayload.resource = res.resource
    if (res.extensions) {
      // Echo back what the server advertised. A client may add to the info it
      // received, but must not drop or overwrite any of it.
      request.paymentPayload.extensions = res.extensions
    }
    return request
  }

  async buildX402Request(
    req: PaymentRequirements,
    x402Version: number
  ): Promise<X402Request<IcpPayload>> {
    if (req.network !== this.network) {
      throw new Error(
        `Network ${req.network} not supported by Anda x402 facilitator`
      )
    }

    const method = assetTransferMethodOf(req)
    if (method !== ASSET_TRANSFER_METHOD_ICRC2) {
      throw new Error(
        `extra.${EXTRA_ASSET_TRANSFER_METHOD} ${method} not supported by Anda x402 facilitator, expected ${ASSET_TRANSFER_METHOD_ICRC2}`
      )
    }

    const flow = paymentFlowOf(req)
    if (!SUPPORTED_PAYMENT_FLOWS.includes(flow)) {
      throw new Error(
        `extra.${EXTRA_PAYMENT_FLOW} ${flow} not supported by Anda x402 facilitator`
      )
    }

    const [info, nonce] = await Promise.all([this.getInfo(), this.nextNonce()])
    const supportedPayment = info.supportedPayments.find(
      (sp) =>
        sp.network === req.network &&
        sp.scheme === req.scheme &&
        sp.x402Version === x402Version
    )

    if (!supportedPayment) {
      throw new Error(
        `Payment scheme ${req.scheme} not supported by Anda x402 facilitator`
      )
    }

    if (!info.supportedAssets[req.asset]) {
      throw new Error(
        `Asset ${req.asset} not supported by Anda x402 facilitator`
      )
    }

    const now = Math.floor(Date.now() / 1000)
    const authorization: IcpPayloadAuthorization = {
      to: req.payTo,
      asset: req.asset,
      value: req.amount,
      expiresAt: (now + req.maxTimeoutSeconds) * 1000,
      nonce: nonce
    }
    const signature = await this.signPayload(authorization)

    return {
      x402Version,
      paymentPayload: {
        x402Version,
        accepted: req,
        payload: {
          signature,
          authorization
        }
      },
      paymentRequirements: req
    }
  }

  /// Fetches the facilitator's `GET /supported` document: the payment kinds and
  /// extensions it implements, plus the `icp:*` signer the payer must approve
  /// as their ICRC-2 spender.
  async getSupported(): Promise<SupportedResponse> {
    const res = await fetch(`${this.endpoint}/supported`, {
      headers: { accept: 'application/json' }
    })
    if (!res.ok) {
      throw new Error(
        `GET ${this.endpoint}/supported failed with ${res.status}: ${await res.text()}`
      )
    }
    return (await res.json()) as SupportedResponse
  }

  async getInfo(): Promise<StateInfo> {
    const _res = await this.#actor.info()
    const res = resultOk(_res)
    return {
      name: res.name,
      supportedPayments: res.supported_payments.map((sp) => ({
        scheme: sp.scheme,
        network: sp.network,
        x402Version: sp.x402_version
      })),
      supportedAssets: Object.fromEntries(
        res.supported_assets.map(([principal, info]) => [
          principal.toText(),
          {
            name: info.name,
            symbol: info.symbol,
            logo: info.logo[0],
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
      scheme: log.scheme,
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
      actor = Actor.createActor<_TOKEN_SERVICE>(tokenIdlFactory, {
        agent: this.#agent,
        canisterId: canisterIdText
      })
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
          token.decimals = Number((value as { 'Nat': bigint }).Nat)
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
    // ICRC-2 `expires_at` is nanoseconds since epoch; build it with BigInt
    // arithmetic so the value is not rounded to the nearest double.
    const expiresAtFloor = BigInt(Date.now() + 60_000) * 1_000_000n
    if (
      allowance.allowance < amount ||
      (expires_at > 0n && expires_at < expiresAtFloor)
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

// `HttpAgent.isLocal()` only recognizes `127.0.0.1`, so `http://localhost:4943`
// would silently be treated as mainnet.
function isLocalHost(host?: string): boolean {
  if (!host) {
    return false
  }

  try {
    const { hostname } = new URL(host)
    return (
      hostname === 'localhost' ||
      hostname.endsWith('.localhost') ||
      hostname === '127.0.0.1' ||
      hostname.endsWith('.127.0.0.1') ||
      hostname === '[::1]'
    )
  } catch {
    return false
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
    if (res.Err instanceof Error) {
      throw res.Err
    }
    // Keep the raw candid variant reachable through `cause`.
    throw new Error(
      typeof res.Err === 'string' ? res.Err : stringifyErr(res.Err),
      { cause: res.Err }
    )
  }

  return res.Ok
}

function stringifyErr(err: unknown): string {
  try {
    // ICRC error variants carry `bigint` fields that `JSON.stringify` rejects.
    return (
      JSON.stringify(err, (_k, v) =>
        typeof v === 'bigint' ? v.toString() : v
      ) ?? String(err)
    )
  } catch {
    return String(err)
  }
}
