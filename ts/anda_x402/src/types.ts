import type { Principal } from '@dfinity/principal'

export interface PaymentRequirements {
  /// Payment scheme identifier (e.g., "exact")
  scheme: 'exact' | 'upto'
  /// Blockchain network identifier (e.g., "icp:1")
  network: string
  /// Required payment amount in atomic token units
  amount: string
  /// Token ledger canister address
  asset: string
  /// Recipient wallet address for the payment
  payTo: string
  /// Maximum time allowed for payment completion in seconds
  maxTimeoutSeconds: number
  /// Scheme-specific additional information.
  extra?: Record<string, unknown>
}

export interface ResourceInfo {
  /// the protected resource, e.g., URL of the resource endpoint
  url: string
  /// Human-readable description of the resource
  description?: string
  /// MIME type of the expected response
  mimeType?: string
}

///  Describes additional extension data for x402 payment.
export interface Extensions {
  info: Record<string, unknown>
  schema: Record<string, unknown>
}

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
  paymentPayload: PaymentPayload<T>
  paymentRequirements: PaymentRequirements
}

export interface VerifyResponse {
  isValid: boolean
  payer?: string
  invalidReason?: string
}

export interface SettleResponse {
  success: boolean
  errorReason?: string
  transaction: string
  network: string
  payer?: string
}

export interface SupportedKind {
  x402Version: number
  scheme: string
  network: string
  extra?: Record<string, unknown>
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

const locale = new Intl.Locale(globalThis?.navigator?.language || 'en')

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
