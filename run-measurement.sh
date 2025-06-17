#!/usr/bin/env sh

OS=macos
PROC=m4max
LOGF=./results/$OS-$PROC-`date -u "+%Y-%m-%d_%H-%M-%S"`.txt; ./target/release/measure-clocks > $LOGF 2>&1 ; cat $LOGF
