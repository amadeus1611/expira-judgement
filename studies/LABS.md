# Proposed sandbox labs and system progression

The owner wants to learn code from the beginning and build security labs as their
concepts become relevant. Longer term, the goal is an AI-assisted system for
authorized vulnerability/exploit checking, using Claude or GPT. No lab or production
system is built, deployed or validated by this memory setup.

## Progression

| Stage | Proposed work | Evidence to retain |
|---|---|---|
| Basic code and computer use | Terminal/files/permissions, variables, conditions, loops, functions and simple Python tests | Small programs the learner can run, explain, modify and debug |
| Network observation | Local client/server communication, DNS examples and packet inspection | Captures or outputs tied to a known lab setup, with the learner's interpretation |
| Security concepts | Authentication, access control, cryptographic tool usage, logs, hardening and recovery | Configurations, actual observations and explained security assumptions |
| Vulnerability practice | Intentionally vulnerable applications in a dedicated sandbox, with controlled findings and remediation | Reproducible cases, confirmed findings, fixes and repeat checks |
| System prototype | A separate application combining approved tools, source/tool evidence, EXPIRA checks and model-assisted explanations | Versioned components, tests, scope records, report validation and operational failures |
| Deployment evaluation | Evaluate a reviewed prototype on explicitly authorized targets | Deployment evidence and independently reviewed release criteria |

Choose exact tools when a lab is scheduled, checking their documentation and licence.
Record platform constraints first. Use the appropriate subject's `labs/` for actual artifacts when they exist;
include a README with setup, scope, execution, observations, reset and cleanup.

## Roles in the eventual system

- Deterministic tools and reproducible observations provide evidence about targets.
- Claude/GPT can assist with explanations, code, triage and proposed investigation
  steps. Generated findings still need evidence and review.
- EXPIRA consumes claims plus source/tool evidence through its existing interface.
  Its finding is part of review, rather than a substitute for confirming a vulnerability.
- Executable checks have defined target scope, permissions and limits, with logs and
  a way to stop or reset them. Model text does not independently authorize operations.

Keep model connections and this application separate from judgement/threshold code.
Choose authentication and data handling when an actual connection is needed; no model
API, credentials, paid calls or production transport is configured by this plan.

## Separate readiness gates

Exam readiness requires demonstrated learning and results on reviewed practice material.
System readiness requires its own evidence: reproducible findings and error handling,
tested target boundaries, protected credentials/data, reviewed model/tool behaviour,
and operating procedures for deployment, monitoring and recovery. Define measurable
release criteria for the intended use before evaluating it; do not invent pass rates,
false-positive rates or production guarantees.

The goal is to build an increasingly useful system during learning. Whether it can
enter production by exam time remains unestablished and depends on observed results.
