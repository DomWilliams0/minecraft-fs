#!/bin/bash -e

flatc --rust --kotlin command.fbs response.fbs

rm -fv ../ipc/src/generated/*_generated.rs
mv -v command_generated.rs response_generated.rs ../ipc/src/generated/
rustfmt ../ipc/src/generated/*_generated.rs

rm -fv ../plugin/src/main/java/ms/domwillia/mcfs/generated/*kt
mv -v MCFS/* ../plugin/src/main/java/ms/domwillia/mcfs/generated/
rmdir MCFS

