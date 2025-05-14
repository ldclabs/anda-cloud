# `anda_registry_canister`

## Features

## Demo

Try it online: https://a4gq6-oaaaa-aaaab-qaa4q-cai.raw.icp0.io/?id=lfcwh-piaaa-aaaap-an2fa-cai

## Quick Start

### Local Deployment

Deploy the canister:
```bash
# dfx canister create --specified-id lfcwh-piaaa-aaaap-an2fa-cai anda_registry_canister
dfx deploy anda_registry_canister --argument "(opt variant {Init =
  record {
    name = \"LDC Labs\";
    challenge_expires_in_ms = 3600000;
    governance_canister = null;
  }
})"
```

## License
Copyright © 2024-2025 [LDC Labs](https://github.com/ldclabs).

`ldclabs/anda-cloud` is licensed under the MIT License. See [LICENSE](../../LICENSE) for the full license text.