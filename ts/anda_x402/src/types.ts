import { Principal } from '@dfinity/principal'

export interface PaymentRequirements {
  /// Payment scheme identifier (e.g., "exact")
  scheme: 'exact' | 'upto'
  /// Blockchain network identifier (e.g., "icp-druyg-tyaaa-aaaaq-aactq-cai")
  network: string
  /// Required payment amount in atomic token units
  maxAmountRequired: string
  /// Token ledger canister address
  asset: string
  /// Recipient wallet address for the payment
  payTo: string
  /// the protected resource, e.g., URL of the resource endpoint
  resource: string
  /// Human-readable description of the resource
  description: string
  /// MIME type of the expected response
  mimeType?: string
  /// JSON schema describing the response format
  outputSchema?: object
  /// Maximum time allowed for payment completion in seconds
  maxTimeoutSeconds: number
  /// Scheme-specific additional information.
  extra?: object
}

export interface PaymentRequirementsResponse {
  x402Version: number
  error: string
  accepts: PaymentRequirements[]
}

export interface PaymentPayload {
  x402Version: number
  scheme: string
  network: string
  payload: object
}

export interface IcpPayload {
  /// The signature of the `authorization` signed by Internet Identity
  signature: string
  /// Parameters required for payment.
  authorization: IcpPayloadAuthorization
}

export interface IcpPayloadAuthorization {
  /// Payment scheme identifier
  scheme: 'exact' | 'upto'
  /// ICRC2 token ledger canister address
  asset: string
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

export interface X402Request {
  paymentPayload: PaymentPayload
  paymentRequirements: PaymentRequirements
}

export interface VerifyResponse {
  isValid: boolean
  payer: string
  invalidReason?: string
}

export interface SettleResponse {
  success: boolean
  errorReason?: string
  transaction: string
  network: string
  payer: string
}

export interface SupportedPaymentKind {
  scheme: string
  network: string
  x402Version: number
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

export function parseNetwork(network: string): Principal {
  if (network.startsWith('icp-')) {
    return Principal.fromText(network.slice(4))
  }

  throw new Error(`Unsupported network format: ${network}`)
}

export function toNetwork(principal: Principal): string {
  return `icp-${principal.toText()}`
}

/// Parses a transaction string in the format "log_id:asset_canister:block_idx"
export function parseTransaction(
  transaction: string
): [bigint, Principal, bigint] {
  const parts = transaction.split(':')
  if (parts.length === 3) {
    return [BigInt(parts[0]), Principal.fromText(parts[1]), BigInt(parts[2])]
  }

  throw new Error(`Unsupported transaction format: ${transaction}`)
}

export interface StateInfo {
  name: string
  supportedPayments: SupportedPaymentKind[]
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
  const decimalVal = 10n ** BigInt(decimals)
  const integerPart = amount / decimalVal
  const fractionalPart = amount % decimalVal
  const val = Number(integerPart) + Number(fractionalPart) / Number(decimalVal)
  return new Intl.NumberFormat(locale, {
    minimumFractionDigits: 0,
    maximumFractionDigits: maxDigits,
    roundingMode: 'trunc'
  } as Intl.NumberFormatOptions).format(val)
}
