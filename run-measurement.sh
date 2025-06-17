#!/usr/bin/env sh

OS=unknownos
PROC=unknownproc
LOGF=./results/$OS-$PROC-`date -u "+%Y-%m-%d_%H-%M-%S"`.txt; ./target/release/measure-clocks | tee $LOGF 2>&1
