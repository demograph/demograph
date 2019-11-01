#!/usr/bin/env bash

if [[ -n $1 ]]; then
    CMD=test
fi

RUST_LOG="warn,demograph=trace,websock=trace,hyper=trace,tokio_reactor=trace" cargo watch -x fmt -x $CMD -x run -i data # -i *.log -i *.md