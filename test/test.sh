#!/bin/sh
arg1="$1"
[ "$#" -eq 1 ] || {
	echo 1>&2 usage: $0 \<n\>, where \<n\>.asm exists
	exit 1
}
[ -f "$arg1.asm" ] || {
	echo 1>&2 $arg1: no such file or directory
	exit 1
}

rm *.o .*.o .o
cd ..
cargo build
cd test
../target/debug/asm "$arg1.asm" > "$arg1.o"
echo ========Cod de asm========
od -An -v -t x4 -w4 "$arg1.o"
echo =======Executie sim=======
../target/debug/main "$arg1.o"

