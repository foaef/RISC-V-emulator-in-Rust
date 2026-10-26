#!/bin/sh
cd ..
cargo build
cd test
../target/debug/asm test.asm > test.o
od -An -v -t x4 -w4 test.o
