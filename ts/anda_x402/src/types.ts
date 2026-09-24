import type { Principal } from '@icp-sdk/core/principal'

/// The x402 protocol version implemented by this SDK.
export const X402_VERSION = 2

/// CAIP-2 identifier of the Internet Computer mainnet.
export const NETWORK_ICP = 'icp:1'

/// Reserved `PaymentRequirements.extra` key naming the asset transfer method
/// (x402 v2, section 6.1).
export const EXTRA_ASSET_TRANSFER_METHOD = 'assetTransferMethod'

/// Reserved `PaymentRequirements.extra` key naming the payment flow
/// (x402 v2, section 6.1).
export const EXTRA_PAYMENT_FLOW = 'paymentFlow'

/// The only asset transfer method of the `icp` mechanism: the payer grants the
/// facilitator an ICRC-2 allowance which it pulls with `icrc2_transfer_from`.
/// It is also the mechanism default, so `extra.assetTransferMethod` may be
/// omitted.
export const ASSET_TRANSFER_METHOD_ICRC2 = 'icrc2'

/// When settlement happens relative to the resource server executing the paid
/// request (x402 v2, section 6.1).
///
/// - `authorization`: verify -> resource -> settle -> respond. The default.
/// - `upfront`: settle -> resource -> respond, when the resource needs
///   finality before it runs.
/// - `escrow`: settle -> resource -> settle -> respond. Not supported by the
///   `icp` mechanism, which has no two-step settlement.
export type PaymentFlow = 'authorization' | 'upfront' | 'escrow'

/// The payment flows the `icp` mechanism can serve, most preferred first.
/// A client must not build a payment for a flow it does not recognize, and
/// should prefer `authorization` when a resource offers both.
export const SUPPORTED_PAYMENT_FLOWS: readonly PaymentFlow[] = [
  'authorization',
  'upfront'
]

export interface PaymentRequirements {
  /// Payment scheme identifier (e.g., "exact")
  scheme: 'exact' | 'upto'
  /// Blockchain network identifier in CAIP-2 format (e.g., "icp:1")
  network: string
  /// Required payment amount in atomic token units
  amount: string
  /// Token ledger canister address
  asset: string
  /// Recipient wallet address for the payment
  payTo: string
  /// Maximum time allowed for payment completion in seconds
  maxTimeoutSeconds: number
  /// Additional information. The `assetTransferMethod` and `paymentFlow` keys
  /// are reserved by the protocol; the rest are scheme-specific.
  extra?: Record<string, unknown>
}

/// Reads the reserved `extra.paymentFlow` key, which defaults to
/// `authorization` when absent. Throws when the key holds something that is
/// not a payment flow, because guessing would mean paying under an ordering
/// the client did not agree to.
export function paymentFlowOf(req: PaymentRequirements): PaymentFlow {
  const flow = req.extra?.[EXTRA_PAYMENT_FLOW]
  if (flow == null) {
    return 'authorization'
  }
  if (flow === 'authorization' || flow === 'upfront' || flow === 'escrow') {
    return flow
  }
  throw new Error(`Unknown extra.${EXTRA_PAYMENT_FLOW}: ${String(flow)}`)
}

/// Reads the reserved `extra.assetTransferMethod` key, which defaults to
/// `icrc2` when absent.
export function assetTransferMethodOf(req: PaymentRequirements): string {
  const method = req.extra?.[EXTRA_ASSET_TRANSFER_METHOD]
  if (method == null) {
    return ASSET_TRANSFER_METHOD_ICRC2
  }
  if (typeof method !== 'string') {
    throw new Error(
      `extra.${EXTRA_ASSET_TRANSFER_METHOD} must be a string, got: ${String(method)}`
    )
  }
  return method
}

/// Whether the `icp` mechanism can serve this requirement's reserved keys.
/// Unknown values make the entry unusable rather than throwing, so a client
/// can skip it and pick another `accepts[]` entry.
export function isSupportedRequirement(req: PaymentRequirements): boolean {
  try {
    return (
      assetTransferMethodOf(req) === ASSET_TRANSFER_METHOD_ICRC2 &&
      SUPPORTED_PAYMENT_FLOWS.includes(paymentFlowOf(req))
    )
  } catch {
    return false
  }
}

export interface ResourceInfo {
  /// the protected resource, e.g., URL of the resource endpoint
  url: string
  /// Human-readable description of the resource
  description?: string
  /// MIME type of the expected response
  mimeType?: string
  /// Human-readable name of the service hosting the resource.
  /// Printable ASCII, at most 32 characters.
  serviceName?: string
  /// Topical tags for the service, used for discovery filtering.
  /// At most 5 entries, each constrained like `serviceName`.
  tags?: string[]
  /// Absolute `https`/`http` URL to an icon representing the service.
  /// At most 2048 characters.
  iconUrl?: string
}

/// One entry of an `Extensions` map.
export interface Extension {
  /// Extension-specific data provided by the server
  info: Record<string, unknown>
  /// JSON Schema defining the expected structure of `info`
  schema: Record<string, unknown>
}

/// Protocol extensions data, keyed by extension identifier.
///
/// Servers advertise the extensions they support in `PaymentRequired` and
/// clients echo them back in `PaymentPayload`. A client must return at least
/// the info it received; it may add to it but must not drop or overwrite it.
export type Extensions = Record<string, Extension>

export interface PaymentRequired {
  x402Version: number
  error?: string
  resource: ResourceInfo
  accepts: PaymentRequirements[]
  extensions?: Extensions
}

export interface PaymentPayload<T> {
  x402Version: number
  resource?: ResourceInfo
  accepted: PaymentRequirements
  payload: T
  extensions?: Extensions
}

export interface IcpPayload {
  /// The signature of the `authorization` signed by Internet Identity
  signature: string
  /// Parameters required for payment.
  authorization: IcpPayloadAuthorization
}

export interface IcpPayloadAuthorization {
  /// Recipient's wallet address
  to: string
  /// Token ledger canister the payment is drawn from. It must equal the
  /// requirements' `asset`; signing it keeps the recipient from settling the
  /// authorized `value` on another ledger the payer approved.
  asset: string
  /// Payment amount in atomic units.
  /// For `exact` scheme, this is the exact amount to be transferred.
  /// For `upto` scheme, this is the maximum amount that can be transferred.
  value: string
  /// Expiration time of the authorization in milliseconds since epoch
  expiresAt: number
  /// A self-incrementing number and should be used to prevent replay attacks.
  nonce: number
}

export interface X402Request<T> {
  x402Version: number
  paymentPayload: PaymentPayload<T>
  paymentRequirements: PaymentRequirements
}

export interface VerifyResponse {
  /// Indicates whether the payment authorization is valid
  isValid: boolean
  /// Address of the payer's wallet
  payer?: string
  /// Reason for invalidity (omitted if valid)
  invalidReason?: string
  /// Protocol extensions data, keyed by extension identifier
  extensions?: Extensions
  /// Scheme-specific additional data
  extra?: Record<string, unknown>
}

export interface SettleResponse {
  /// Indicates whether the payment settlement was successful
  success: boolean
  /// Error reason if settlement failed (omitted if successful)
  errorReason?: string
  /// Ledger transaction ID. Empty when no transaction was committed, which a
  /// `settlement_pending` reason is the one error not allowed to be.
  transaction: string
  /// Blockchain network identifier in CAIP-2 format
  network: string
  /// Address of the payer's wallet
  payer?: string
  /// The actual amount settled in atomic units (omitted if not applicable)
  amount?: string
  /// Protocol extensions data, keyed by extension identifier
  extensions?: Extensions
}

export interface SupportedKind {
  x402Version: number
  scheme: string
  network: string
  extra?: Record<string, unknown>
}

/// The response of the facilitator's `GET /supported` endpoint.
export interface SupportedResponse {
  /// Supported payment kinds
  kinds: SupportedKind[]
  /// Extension identifiers the facilitator has implemented
  extensions: string[]
  /// CAIP-2 patterns (e.g. `icp:*`) mapped to the facilitator's signer
  /// addresses. On ICP the signer is the canister the payer must approve as
  /// their ICRC-2 spender.
  signers: Record<string, string[]>
}

export interface AssetInfo {
  name: string
  symbol: string
  decimals: number
  transferFee: bigint
  paymentFee: bigint
  logo?: string
}

export interface PaymentLogInfo {
  id: bigint
  to: string
  fee: string
  asset: string
  value: string
  scheme: string
  from: string
  nonce: bigint
  timestamp: bigint
  expiresAt: bigint
}

export interface TokenInfo {
  name: string
  symbol: string
  decimals: number
  fee: bigint
  logo: string
  canisterId: Principal
}

export interface StateInfo {
  name: string
  supportedPayments: SupportedKind[]
  supportedAssets: Record<string, AssetInfo>
  totalWithdrawnFees: Record<string, bigint>
  totalCollectedFees: Record<string, bigint>
  governanceCanister?: string
}

// Read `navigator` off `globalThis` so the module also type-checks for
// consumers whose `lib` does not include DOM (Node exposes it since v21).
const locale = new Intl.Locale(
  (globalThis as { navigator?: { language?: string } }).navigator?.language ||
    'en'
)

export function formatAmount(
  amount: bigint,
  decimals: number,
  maxDigits: number = 6
): string {
  return new Intl.NumberFormat(locale, {
    minimumFractionDigits: 0,
    maximumFractionDigits: maxDigits,
    roundingMode: 'trunc'
  } as Intl.NumberFormatOptions).format(toDecimalString(amount, decimals))
}

/// Renders an atomic token amount as an exact decimal string.
/// Going through `Number` would round amounts that need more than 53 bits,
/// e.g. `0.999999999999999999` of an 18 decimals token would become `1`.
function toDecimalString(
  amount: bigint,
  decimals: number
): Intl.StringNumericLiteral {
  if (decimals <= 0) {
    return amount.toString() as Intl.StringNumericLiteral
  }

  const sign = amount < 0n ? '-' : ''
  const digits = (amount < 0n ? -amount : amount)
    .toString()
    .padStart(decimals + 1, '0')
  const point = digits.length - decimals
  return `${sign}${digits.slice(0, point)}.${digits.slice(
    point
  )}` as Intl.StringNumericLiteral
}
