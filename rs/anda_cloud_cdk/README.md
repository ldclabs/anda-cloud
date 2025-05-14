# `anda_cloud_cdk`
![License](https://img.shields.io/crates/l/anda_cloud_cdk.svg)
[![Crates.io](https://img.shields.io/crates/d/anda_cloud_cdk.svg)](https://crates.io/crates/anda_cloud_cdk)
[![Test](https://github.com/ldclabs/anda-cloud/actions/workflows/test.yml/badge.svg)](https://github.com/ldclabs/anda-cloud/actions/workflows/test.yml)
[![Docs.rs](https://img.shields.io/docsrs/anda_cloud_cdk?label=docs.rs)](https://docs.rs/anda_cloud_cdk)
[![Latest Version](https://img.shields.io/crates/v/anda_cloud_cdk.svg)](https://crates.io/crates/anda_cloud_cdk)

`anda_cloud_cdk` is a Canister Development Kit for the Anda Cloud, providing essential structures and utilities for building and interacting with the Anda network ecosystem on the Internet Computer Protocol (ICP).

## Overview

This crate provides the core data structures, validation logic, and cryptographic utilities needed to:

- Register and manage AI agents in the Anda network
- Implement the agent challenge-response protocol for health verification
- Interact with Anda Registry Canisters
- Support Trusted Execution Environment (TEE) attestation

## Key Features

- **Agent Management**: Comprehensive structures for agent registration and information management, supporting ANDA, A2A, MCP, and other protocols
- **Challenge Protocol**: Implementation of the agent health verification system
- **Payment Protocol**: Support for the X402 payment protocol
- **TEE Support**: Structures for Trusted Execution Environment attestation

## Documentation

For detailed documentation of all structures and functions, please visit [docs.rs/anda_cloud_cdk](https://docs.rs/anda_cloud_cdk).

### Challenge Process

The complete challenge process is as follows:
1. The challenger obtains the agent's challenge code from the Registry Canister.
   For first-time registration, the challenge code is [0u8; 32].
2. The challenger obtains the agent's latest information through its protocol
   and generates a ChallengeRequest.
3. The challenger signs the ChallengeRequest with their ICP identity and sends it to the agent.
4. Upon receiving the challenger's request, the agent confirms the information
   and signs it with its own ICP identity.
5. If the agent is running in a TEE environment, it also generates TEE information,
   including an attestation containing this challenge information.
6. The challenger sends the ChallengeEnvelope returned by the agent to the Registry Canister.
7. The Registry Canister verifies the ChallengeEnvelope and updates the agent's status.
8. Since the challenge code is updated after each successful challenge, only the first
   challenger for a given challenge code can succeed. Requests from other challengers
   for the same challenge code will be invalid.

## Related Projects

- [Anda Registry Canister](https://github.com/ldclabs/anda-cloud/tree/main/rs/anda_registry_canister): The canister implementation for the Anda agent registry
- [Anda Protocol](https://github.com/ldclabs/anda): The Anda Network Decentralized Agent protocol specification
- [ICAuth](https://github.com/ldclabs/ic-auth): The Internet Computer identity based web authentication

## License
Copyright © 2024-2025 [LDC Labs](https://github.com/ldclabs).

`ldclabs/anda-cloud` is licensed under the MIT License. See [LICENSE](../../LICENSE) for the full license text.