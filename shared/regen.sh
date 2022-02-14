#!/bin/bash -e

flatc --rust --gen-all --gen-onefile --include-prefix generated all.fbs
flatc --kotlin --gen-all --include-prefix generated all.fbs
echo flatc is done

rm -fv ../ipc/src/generated/*_generated.rs
mv -v *_generated.rs ../ipc/src/generated/
rustfmt ../ipc/src/generated/*_generated.rs
echo rust is done

rm -fv ../plugin/src/main/java/ms/domwillia/mcfs/generated/*kt
mv -v MCFS/* ../plugin/src/main/java/ms/domwillia/mcfs/generated/
rmdir MCFS
echo kotlin is done

echo complete
