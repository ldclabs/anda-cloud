#!/usr/bin/env bash

# Load the environment variables
source "$(pwd)"/proposals/env.sh

quill sns make-proposal --canister-ids-file ./sns_canister_ids.json --pem-file $PROPOSAL_PEM_FILE $PROPOSAL_NEURON_ID --proposal '(
    record {
        title = "Register anda_cloud_website Canister";
        url = "https://github.com/ldclabs/anda-cloud/tree/main/ts/anda_cloud_website";
        summary = "Anda Cloud Website: https://anda.ai/";
        action = opt variant {
            RegisterDappCanisters = record {
                canister_ids = vec {principal "lxeb6-dyaaa-aaaap-an2ga-cai"};
            }
        };
    }
)' > proposal-message.json

# quill send proposal-message.json