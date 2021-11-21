#!/usr/bin/env bash

set -eou pipefail

./target/release/rewrk -d 10s -c 20 -t 2 -h https://www.cloudflare.com/rate-limit-test/

