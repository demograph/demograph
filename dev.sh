#!/usr/bin/env bash

CMDS=
for CMD in "$@"
do
    CMDS="$CMDS -x $CMD "
done

cargo watch -x fmt $CMDS