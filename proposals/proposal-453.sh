#!/usr/bin/env bash

# Load the environment variables
source "$(pwd)"/proposals/env.sh

# build and get batch_id, evidence:
# dfx deploy anda_cloud_website --ic --by-proposal

export BLOB="$(didc encode --format blob '(record {batch_id=19:nat; evidence=blob "\cb\cc\a5\4b\a8\12\58\e3\52\32\cf\94\da\de\08\7f\f4\d9\f7\8c\f5\b1\1c\0a\22\53\4e\81\fd\ec\f6\ed"})')"

quill sns make-proposal --canister-ids-file ./sns_canister_ids.json --pem-file $PROPOSAL_PEM_FILE $PROPOSAL_NEURON_ID --proposal "(
    record {
        title = \"Execute commit_proposed_batch() to release anda_cloud_website v0.3.0\";
        url = \"https://anda.ai/\";
        summary = \"This proposal executes commit_proposed_batch() on lxeb6-dyaaa-aaaap-an2ga-cai to release anda_cloud_website v0.3.0.\n\n1. chore: add Anda x402 facilitator.\";
        action = opt variant {
            ExecuteGenericNervousSystemFunction = record {
                function_id = 1200 : nat64;
                payload = ${BLOB};
            }
        };
    }
)" > proposal-message.json

# quill send proposal-message.json