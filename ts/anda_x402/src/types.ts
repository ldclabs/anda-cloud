import { Principal } from '@dfinity/principal'

export interface PaymentRequirements {
  scheme: 'exact' | 'upto'
  network: string
  maxAmountRequired: string
  asset: string
  payTo: string
  resource: string
  description: string
  mimeType?: string
  outputSchema?: object
  maxTimeoutSeconds: number
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
  signature: string
  authorization: IcpPayloadAuthorization
}

export interface IcpPayloadAuthorization {
  scheme: 'exact' | 'upto'
  asset: string
  to: string
  value: string
  expiresAt: number
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
  decimals: number
  transferFee: bigint
  paymentFee: bigint
  symbol: string
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

export function parseTransaction(transaction: string): [Principal, bigint] {
  const parts = transaction.split(':')
  if (parts.length === 2) {
    return [Principal.fromText(parts[0]), BigInt(parts[1])]
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
