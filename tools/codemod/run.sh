#!/usr/bin/env bash
echo "OBSOLETE: pemindahan model->ops sudah selesai dan diedit manual; menjalankan skrip ini akan menimpanya." >&2; exit 1
# Jalankan dari root repo: tools/codemod/run.sh  (reset model -> transformasi -> perbaikan visibilitas)
set -e
export PATH=/home/claude/rs/bin:$PATH
C=crates/oxcord/src; M=$C/model
git checkout -q $C/model && rm -rf $C/ops
python3 - <<'P'
p='crates/oxcord/src/model/mod.rs'
s=open(p).read()
s=s.replace('    pub(crate) use crate::internal::prelude::*;\n}','    #[cfg(feature = "model")]\n    #[doc(hidden)]\n    pub use crate::ops::*;\n    pub(crate) use crate::internal::prelude::*;\n}',1)
open(p,'w').write(s)
P
(cd tools/codemod && cargo build -q --release)
tools/codemod/target/release/oxcord-codemod apply $C/ops $M $(find $M -name '*.rs' | sort) >/dev/null
# visibilitas: model::utils dan submodul dibuka ke crate
sed -i -E 's/^mod utils;/pub(crate) mod utils;/' $M/mod.rs
sed -i -E 's/^pub\(super\) /pub(crate) /; s/^(async )?(const )?fn /pub(crate) \1\2fn /; s/^enum /pub(crate) enum /; s/^struct /pub(crate) struct /; s/^mod /pub(crate) mod /' $M/utils.rs
for f in $M/application/mod.rs $M/channel/mod.rs $M/guild/mod.rs; do sed -i -E 's/^((#\[cfg[^]]*\]\s*)?)mod ([a-z_]+);/\1pub(crate) mod \3;/' $f; done
sed -i '193s/^    id: TargetId,/    pub(crate) id: TargetId,/' $M/channel/mod.rs
