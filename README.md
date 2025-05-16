# Anda Cloud

Anda Cloud is a decentralized AI Agent infrastructure built on ICP and TEE.

## Overview

Anda Cloud is a decentralized AI Agent infrastructure built on the Internet Computer Protocol (ICP) and Trusted Execution Environment (TEE). It enables an open, secure, trustworthy, and highly collaborative AI Agent network where agents can register, discover each other, transact securely, and interact via standardized protocols.

## Core Services

### AI Agent Registration & Discovery: ([`anda_registry_canister`](./rs/anda_registry_canister))

A fully on-chain AI agents registry & discovery service deployed as an ICP smart contract.

**Key Features**:

- Registration and discovery of AI agents across multiple protocols
- Health monitoring through a challenge-based verification system
- Support for Trusted Execution Environment (TEE) attestation
- Global unique handle registration with name service provided by [dMsg.net](https://dMsg.net)

### [WIP] Enhanced AI Agent Discovery (`anda_discovery_service`)

A TEE-based intelligent service that enhances agent discovery, monitoring, and interaction.

**Key Features**:

- Supports MCP (Model Context Protocol), A2A (Agent2Agent protocol), and ANDA (Autonomous Networked Decentralized Agent) protocols.
- Assists agents in registering with `anda_registry_canister`.
- Challenge agents health (availability, responsiveness, reputation).
- Provides advanced search & filtering for agents based on capabilities, pricing, and trust metrics.
- Facilitates secure, privacy-preserving agent matching.

### [WIP] AI Agent Payment Service (`anda_payment_service`)

A TEE-based payment & settlement service supporting the X402 payment protocol.

**Key Features**:

- Enables instant transactions & true micropayments between AI Agents.
- Supports muiltiple blockchain tokens (e.g., ICP, BNB, SOL, ETH) and stablecoins.
- Ensures confidential transaction details (TEE-guaranteed privacy).
- Provides dispute resolution & escrow mechanisms for multi-agent agreements.

### [WIP] AI Agent Gateway Service (`anda_gateway_service`)

A TEE-based gateway that bridges external AI Agents (using MCP/A2A) with the ANDA ecosystem.

**Key Features**:

- Integrates Internet Identity (II) for non-ANDA agents.
- Enables payment capabilities for agents that natively lack them (via `anda_payment_service`).
- Provides a unified interface for external agents to interact with the ANDA ecosystem.
- Facilitates seamless integration of external agents into the ANDA network.

## Related Projects

- [Anda Protocol & Framework](https://github.com/ldclabs/anda): An AI agent framework built with Rust, powered by ICP and TEEs.
- [IC Auth](https://github.com/ldclabs/ic-auth): The Internet Computer identity based web authentication.
- [IC TEE](https://github.com/ldclabs/ic-tee): Make Trusted Execution Environments (TEEs) work with the Internet Computer.
- [dMsg.net](https://dMsg.net): The world's 1st decentralized end-to-end encrypted messaging application fully running on the Internet Computer blockchain.

## License

Copyright © 2024-2025 [LDC Labs](https://github.com/ldclabs).

`ldclabs/anda-cloud` is licensed under the MIT License. See [LICENSE](./LICENSE) for the full license text.
