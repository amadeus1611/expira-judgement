#!/usr/bin/env bash
# Rebuild every input of the local-cascade bench in one command, from public sources, outside the repo:
#   frozen VitaminC splits + untouched confirmation set, pinned DeBERTa models, layer-1 verdicts, wording-gate rows,
#   and raw judge scores. Each step is skipped when its output already exists, so a rerun resumes.
#
#   tools/rebuild_inputs.sh DATA_DIR [JUDGE ...]        (default judges: xsmall large)
#   python3 -I tools/local_cascade.py DATA_DIR large --gate --confirm
#
# Settings are the frozen ones (docs/bench/2026-10-12-content-word-gate.md): 60 pages for cal/test, --confirm --pages 200,
# source "VitaminC supplied quotation", evidence date and --today 2026-10-11. Data and weights never enter git.
set -euo pipefail
data=${1:?usage: tools/rebuild_inputs.sh DATA_DIR [JUDGE ...]}
shift
judges=("${@:-xsmall large}")
read -r -a judges <<<"${judges[*]}"
repo=$(cd "$(dirname "$0")/.." && pwd)
models=$repo/models
mkdir -p "$data" "$models"

# name -> "hub id@pinned revision"; files fetched: config.json, tokenizer.json, onnx/model.onnx (fp32).
declare -A hub=(
  [xsmall]="cross-encoder/nli-deberta-v3-xsmall@a150876415327c80daeff35ca6f68f5ed8cf5c24"
  [large]="MoritzLaurer/DeBERTa-v3-large-mnli-fever-anli-ling-wanli@b3546ea6b0346eb6f8d5d68b13c7dc6d0376b3d7"
  [roberta]="Xenova/roberta-large-mnli@f44e410851308d901c324c31af38793f50205e91"
  [bart]="Xenova/bart-large-mnli@71a9d2a7c389b1a812afc3cc1668f8c877681b00"
)

step() { printf '[rebuild] %s\n' "$*" >&2; }

if [[ ! -s $data/vitc_test.jsonl ]]; then step "fetch cal/test"; python3 -I "$repo/tools/fetch_general.py" "$data"; fi
if [[ ! -s $data/vitc_confirm.jsonl ]]; then
  step "fetch confirmation"; python3 -I "$repo/tools/fetch_general.py" "$data" --confirm --pages 200
fi
# local_cascade.py reads the confirmation split as "conf".
[[ -e $data/vitc_conf.jsonl ]] || ln -s vitc_confirm.jsonl "$data/vitc_conf.jsonl"

step "build release binaries"
cargo build --release --locked -q --manifest-path "$repo/Cargo.toml" -p ej-bench -p ej-cli
bin=$repo/target/release

for s in cal test conf; do
  if [[ ! -s $data/l1_$s.json ]]; then
    step "layer 1 on $s"
    python3 -I -c '
import json, sys
rows = [json.loads(l) for l in open(sys.argv[1])]
json.dump([{"id": str(i), "text": r["claim"], "evidence": [{"source": "VitaminC supplied quotation",
           "date": "2026-10-11", "quote": r["quote"]}]} for i, r in enumerate(rows)], open(sys.argv[2], "w"))
' "$data/vitc_$s.jsonl" "$data/claims_$s.json"
    # ej check exits 1 when any finding does not pass; that is expected here.
    "$bin/ej" check "$data/claims_$s.json" --today 2026-10-11 >"$data/l1_$s.json" || [[ $? -eq 1 ]]
  fi
  [[ -s $data/vitc_$s.words ]] || { step "wording gate on $s"; "$bin/ej-bench" words "$data/vitc_$s.jsonl" >"$data/vitc_$s.words"; }
done

for j in "${judges[@]}"; do
  spec=${hub[$j]:?unknown judge $j}
  id=${spec%@*} rev=${spec#*@} dir=$models/$j
  if [[ ! -s $dir/model.onnx ]]; then
    step "download $id@$rev"
    mkdir -p "$dir"
    for f in config.json tokenizer.json; do curl -fsSL "https://huggingface.co/$id/resolve/$rev/$f" -o "$dir/$f"; done
    curl -fsSL "https://huggingface.co/$id/resolve/$rev/onnx/model.onnx" -o "$dir/model.onnx.part"
    mv "$dir/model.onnx.part" "$dir/model.onnx"
  fi
  for s in cal test conf; do
    out=$data/vitc_$s.$j.scores
    if [[ ! -s $out ]]; then
      step "score $j on $s"
      "$bin/ej-bench" score "$data/vitc_$s.jsonl" --nli "$dir" --out "$out.part"
      mv "$out.part" "$out"
    fi
  done
done
step "done; run: python3 -I tools/local_cascade.py $data JUDGE --gate --confirm"
