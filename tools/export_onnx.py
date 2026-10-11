#!/usr/bin/env python3
"""Export a Hugging Face sequence-classification checker to the directory layout `ej-nli` loads:
model.onnx, tokenizer.json, config.json.

    python tools/export_onnx.py MODEL_ID OUT_DIR [--labels unsure,supports]

`--labels` writes ej_labels.json (the ruling for each output index) for checkers whose config labels are only "0"/"1",
e.g. MiniCheck, where index 1 is "supported" in its reference code.

Needs torch and transformers (e.g. the LAYA venv). The graph takes input_ids and attention_mask (plus token_type_ids
only if the model uses them) with dynamic batch and sequence axes, and returns logits. fp32; quantise separately and
re-measure, since a careless 8-bit build can wreck accuracy (see docs/bench/2026-10-11-round2-stronger-verifiers.md)."""
import json, os, shutil, sys

import torch
from transformers import AutoConfig, AutoModelForSequenceClassification, AutoTokenizer


def main(model_id, out, labels=None):
    os.makedirs(out, exist_ok=True)
    tok = AutoTokenizer.from_pretrained(model_id, use_fast=True)
    model = AutoModelForSequenceClassification.from_pretrained(model_id).eval()
    AutoConfig.from_pretrained(model_id).to_json_file(os.path.join(out, "config.json"))
    tok.save_pretrained(out)
    enc = tok("A short document for tracing.", "A claim.", return_tensors="pt")
    names = [n for n in ("input_ids", "attention_mask", "token_type_ids") if n in enc]
    if "token_type_ids" in names and getattr(model.config, "type_vocab_size", 0) in (0, None):
        names.remove("token_type_ids")  # model ignores them; keep the graph minimal
    axes = {n: {0: "batch", 1: "seq"} for n in names}
    axes["logits"] = {0: "batch"}
    torch.onnx.export(model, tuple(enc[n] for n in names), os.path.join(out, "model.onnx"), input_names=names,
                      output_names=["logits"], dynamic_axes=axes, opset_version=17)
    if labels:
        with open(os.path.join(out, "ej_labels.json"), "w") as f:
            json.dump(labels.split(","), f)
    for f in os.listdir(out):  # keep only what ej-nli reads
        if f not in ("model.onnx", "model.onnx.data", "tokenizer.json", "config.json", "ej_labels.json"):
            p = os.path.join(out, f)
            shutil.rmtree(p) if os.path.isdir(p) else os.remove(p)
    print("exported", model_id, "->", out, "inputs:", names)


if __name__ == "__main__":
    args = sys.argv[1:]
    labels = args[args.index("--labels") + 1] if "--labels" in args else None
    pos = [a for i, a in enumerate(args) if a != "--labels" and (i == 0 or args[i - 1] != "--labels")]
    if len(pos) != 2:
        raise SystemExit(__doc__)
    main(pos[0], pos[1], labels)
