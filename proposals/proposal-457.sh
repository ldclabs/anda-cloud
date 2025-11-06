#!/usr/bin/env bash

# Load the environment variables
source "$(pwd)"/proposals/env.sh

# build and get batch_id, evidence:
# dfx deploy anda_cloud_website --ic --by-proposal

export BLOB="$(didc encode --format blob '(record {batch_id=21:nat; evidence=blob "\73\68\f2\ad\f5\7f\01\f6\f4\1b\68\fb\ed\90\47\68\80\2c\fc\37\62\44\7c\ff\94\7e\e4\15\11\90\ae\03"})')"

quill sns make-proposal --canister-ids-file ./sns_canister_ids.json --pem-file $PROPOSAL_PEM_FILE $PROPOSAL_NEURON_ID --proposal "(
    record {
        title = \"Execute commit_proposed_batch() to release anda_cloud_website v0.3.2\";
        url = \"https://anda.ai/\";
        summary = \"This proposal executes commit_proposed_batch() on lxeb6-dyaaa-aaaap-an2ga-cai to release anda_cloud_website v0.3.2.\n\n1. chore: update x402 network id to 'icp'.\";
        action = opt variant {
            ExecuteGenericNervousSystemFunction = record {
                function_id = 1200 : nat64;
                payload = ${BLOB};
            }
        };
    }
)" > proposal-message.json

# quill send proposal-message.json