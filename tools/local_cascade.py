#!/usr/bin/env python3
"""Whole local cascade on VitaminC: layer 1 (`ej check` output l1_{cal,test}.json), then a local judge's scores
(vitc_{split}.{judge}.scores) on what layer 1 left unsure, optionally with the exact-name rule. The threshold is fitted
on calibration (best 95% Wilson lower bound, at least 20 calls) and applied unchanged to test.

    python3 -I tools/local_cascade.py DATA_DIR JUDGE
"""
import json,math,sys,re
D=sys.argv[1]
NOT=set("the a an of in on at to for and or but with by from as is are was were be been his her its their this that these those it he she".split())
def names(c):
    out=[]
    for w in c.split()[1:]:
        w=re.sub(r'^[^\w]+|[^\w]+$','',w)
        if w[:1].isupper() and w.lower() not in NOT: out.append(w.lower())
    return out
def names_ok(c,q):
    words={re.sub(r'^[^\w]+|[^\w]+$','',w).lower() for w in q.split()}
    return all(n in words for n in names(c))
def wil(k,n,z=1.96):
    if n==0: return 0
    p=k/n;d=1+z*z/n;return (p+z*z/(2*n)-z*math.sqrt(p*(1-p)/n+z*z/(4*n*n)))/d
def load(split,judge):
    rows=[json.loads(l) for l in open(f"{D}/vitc_{split}.jsonl")]
    l1={int(x["claim_id"]):x["verdict"] for x in json.load(open(f"{D}/l1_{split}.json"))}
    sc={}
    for l in open(f"{D}/vitc_{split}.{judge}.scores"):
        s=json.loads(l); sc[s["i"]]=s["p"]
    return rows,l1,sc
def run(rows,l1,sc,t,name_rule):
    k=n=0; blocked_true=0
    for i,r in enumerate(rows):
        v=l1[i]; sup=r["label"]=="supports"
        if v=="backed": n+=1;k+=sup; continue
        if v!="unsure": continue
        if i in sc and sc[i]>=t and (not name_rule or names_ok(r["claim"],r["quote"])):
            n+=1;k+=sup
    return k,n
judge=sys.argv[2]
cal=load("cal",judge); test=load("test",judge)
for rule in (False,True):
    best=None
    for t in sorted(set(cal[2].values())):
        k,n=run(*cal,t,rule)
        if n<20: continue
        lb=wil(k,n)
        if best is None or lb>best[0]: best=(lb,t,k,n)
    lb,t,k,n=best
    tk,tn=run(*test,t,rule)
    print(f"{judge} name_rule={rule}: threshold {t:.4f} | cal {k}/{n} lb {lb:.4f} | TEST {tk}/{tn} = {tk/tn:.4f}, lb {wil(tk,tn):.4f}")
