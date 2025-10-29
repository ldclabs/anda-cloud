#!/usr/bin/env bash

# Load the environment variables
source "$(pwd)"/proposals/env.sh

# build and get batch_id, evidence:
# dfx deploy anda_cloud_website --ic --by-proposal

export BLOB="$(didc encode --format blob '(record {batch_id=20:nat; evidence=blob "\dc\59\48\af\94\7d\2c\9b\4c\b9\e8\32\dc\89\a0\78\6a\cc\f2\81\50\f9\18\98\63\d5\89\ff\df\8f\4a\83"})')"

quill sns make-proposal --canister-ids-file ./sns_canister_ids.json --pem-file $PROPOSAL_PEM_FILE $PROPOSAL_NEURON_ID --proposal "(
    record {
        title = \"Execute commit_proposed_batch() to release anda_cloud_website v0.3.1\";
        url = \"https://anda.ai/\";
        summary = \"This proposal executes commit_proposed_batch() on lxeb6-dyaaa-aaaap-an2ga-cai to release anda_cloud_website v0.3.1.\n\n1. chore: update x402 content.\";
        action = opt variant {
            ExecuteGenericNervousSystemFunction = record {
                function_id = 1200 : nat64;
                payload = ${BLOB};
            }
        };
    }
)" > proposal-message.json

# quill send proposal-message.json