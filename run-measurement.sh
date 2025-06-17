#!/usr/bin/env sh

LOGF=./results/unknownos-unknownproc-`date -u "+%Y-%m-%d_%H-%M-%S"`.txt; ./target/release/measure-clocks &> $LOGF ; cat $LOGF
