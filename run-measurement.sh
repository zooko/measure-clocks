#!/usr/bin/env sh

OS=`uname -s`
PROC=`./detect-cpu-details.sh | grep "CPU Model" | cut -d' ' -f3-`
LOGF=./results/$OS-$PROC-`date -u "+%Y-%m-%d_%H-%M-%S"`.txt; ./target/release/measure-clocks | tee $LOGF 2>&1
