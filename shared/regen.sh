#!/bin/bash -e

flatc --rust --kotlin --gen-all --include-prefix generated all.fbs

rm -fv ../ipc/src/generated/*_generated.rs
mv -v *_generated.rs ../ipc/src/generated/
rustfmt ../ipc/src/generated/*_generated.rs

rm -fv ../plugin/src/main/java/ms/domwillia/mcfs/generated/*kt
mv -v MCFS/* ../plugin/src/main/java/ms/domwillia/mcfs/generated/
rmdir MCFS

echo complete
