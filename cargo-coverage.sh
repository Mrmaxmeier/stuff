#!/usr/bin/env bash
set -e

host=$(rustc -vV | awk '/^host/ { print $2 }')
tmpdir=$(mktemp -d)

export RUSTFLAGS="-C instrument-coverage"
export LLVM_PROFILE_FILE="$tmpdir/coverage-%p-%m.profraw"
export PATH="$(rustc --print=target-libdir)/../bin:$PATH"

# cargo run --target $host $@
# TODO: correct bin target?
exe=$(cargo build --release --target $host --message-format=json | jq -r 'select(.executable) | .executable')
echo "Running: $exe ${@:2}"
$exe "${@:2}"

llvm-profdata merge -sparse $tmpdir/*.profraw \
    -o $tmpdir/coverage.profdata
llvm-cov show -Xdemangler=rustfilt $exe \
    -instr-profile=$tmpdir/coverage.profdata \
    -format=html -output-dir=$tmpdir/cov-html

echo "Coverage report: file://$tmpdir/cov-html/index.html"
