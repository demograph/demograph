#!/usr/bin/env bash

if [ -n $1 ]; then
    CMD=test
fi

cargo watch -x fmt -x $CMD -x run -i data # -i *.log -i *.md