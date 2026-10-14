# Current study state

Last updated: 2026-10-14 (Asia/Manila).

## Current position

- Learner: beginner by self-report, available full time.
- Primary goal: `SECPLUS`; next teaching subject: `FOUND`. Subject labels and entry
  points are in [SUBJECTS.md](SUBJECTS.md).
- Phase: orientation and foundation planning; no lesson or learner assessment has
  been completed in this workflow.
- Current lesson resume: [FOUND state](subjects/computing-foundations/STATE.md),
  next proposed `FOUND-001`. No demonstrated foundation skills recorded.
- Exam track: [SECPLUS state](subjects/security-plus/STATE.md); version/objectives
  and date unconfirmed, no scored practice or readiness assessment.
- Future degree: [EE state](subjects/electrical-engineering/STATE.md); specific
  courses not selected. Add other subjects through the registry when requested.
- Labs and production security system: none built in this workflow. Teach code from
  scratch and build labs progressively using `PROCEDURES.md` and `LABS.md`; evaluate
  exam readiness and deployment readiness separately.
- Initial source catalogue: inspected page references in `SOURCES.md`; no lesson
  source snapshots or educational EXPIRA reports have been saved yet.

## Available checking system

- `crates/ej-study` provides a separate CLI/library around production `ej check`.
- Integration PR: https://github.com/amadeus1611/expira-judgement/pull/12
  Last inspected 2026-10-14: open draft, integration commit `a21c196` had all
  reported CI checks successful; later pushes have their own checks.
  Do not assume it is merged; recheck when repository status matters.
- Current recorded integration verification: 135 passing Rust tests, explicit
  real-engine parity, and a full mutation run with zero missed/timeouts. These are
  software checks, not measurements of Security+ question quality or learner ability.
- No local model weights or eligible policy were installed by the integration.
  It forwards an existing model/policy when supplied; default unresolved paraphrases
  can remain `unsure`. Do not label them verified or alter thresholds to settle them.
- Cloud shell setup: `source /workspace/setup/activate.sh` activates the existing
  toolchain/native library. The helper is outside Git; other environments need their
  own setup. Commands and limits are in `crates/ej-study/README.md`.

## Next action

Prepare `FOUND-001` with selected independent sources, a simple demonstration and
a short unscored diagnostic. Save its claims and evidence, run available checks,
review unresolved material, then teach and record the learner's actual responses.
Use the source catalogue as a starting point; it is not a substitute for inspecting
the passages used in the lesson. Establish the learner's device/lab constraints
before choosing installation exercises.

## Progress recording

Use `not started`, `in progress`, `needs review`, `demonstrated` and `review due`.
Advance to `demonstrated` only with recorded explanation, practical work or responses
to unfamiliar questions. Set review dates when an actual session establishes them.
No fixed completion date, pass probability or mastery percentage is established.
Keep detailed progress in each subject's STATE; update this hub's active subject
and next action, together with the affected subject and session log, after study work.
