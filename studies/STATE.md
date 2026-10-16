# Current study state

Last updated: 2026-10-16 (Greek added; FOUND/SECPLUS unchanged since 2026-10-14).

## Current position

- Learner: beginner by self-report, available full time.
- Primary goal: `SECPLUS`; next teaching subject: `FOUND`. Subject labels and entry
  points are in [SUBJECTS.md](SUBJECTS.md).
- Phase: `FOUND-001` in progress; first diagnostic responses reviewed and saved.
  Counting-rule transfer check is pending; no whole-lesson mastery is recorded.
- Current lesson resume: [FOUND state](subjects/computing-foundations/STATE.md),
  current [FOUND-001 v1](subjects/computing-foundations/lessons/FOUND-001-v1/README.md).
  [Actual responses and feedback](subjects/computing-foundations/assessments/FOUND-001-v1-response-01.md)
  record correct byte conversion/program roles and an acknowledged counting correction.
- Exam track: [SECPLUS state](subjects/security-plus/STATE.md); version/objectives
  and date unconfirmed, no scored practice or readiness assessment.
- Future degree: [EE state](subjects/electrical-engineering/STATE.md); specific
  courses not selected. Add other subjects through the registry when requested.
- Labs and production security system: none built in this workflow. Teach code from
  scratch and build labs progressively using `PROCEDURES.md` and `LABS.md`; evaluate
  exam readiness and deployment readiness separately.
- Source catalogue: `SOURCES.md`; the first lesson now has selected source excerpts,
  original EXPIRA reports and manual review. Three definition paraphrases remain
  engine-`unsure` without a local judge; they are not labelled engine-verified.

## Greek track (added 2026-10-16)

- `GREEK`: Mounce BBG 2nd ed., Track Two. Ch. 1–9 and 15 assumed done in Codex;
  ch. 16 and the present active infinitive `-ειν` in progress, quizzes pending.
  Resume: [Greek tracker](subjects/greek/TRACKER.md), [Greek state](subjects/greek/STATE.md).
  Greek sessions run in their own chat; `FOUND`/`SECPLUS` next action below is unchanged.

## Available checking system

- `crates/ej-study` provides a separate CLI/library around production `ej check`.
- Integration PR: https://github.com/amadeus1611/expira-judgement/pull/12
  Merged 2026-10-14, merge commit `d5fad5b`. Integration/memory head `944c64e`
  had both check jobs and one full mutation job successful at merge; a duplicate
  mutation job remained running. Prior local full mutation gate also passed.
  Recheck status when relevant; subsequent study updates have separate PRs/checks.
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

Collect the four-bit transfer response (`Q-FOUND-001-04`), review any remaining
counting gap, then continue the next segment. Establish device/lab constraints before choosing
installation exercises. Follow the saving/merge preference in `PROCEDURES.md`.

## Progress recording

Use `not started`, `in progress`, `needs review`, `demonstrated` and `review due`.
Advance to `demonstrated` only with recorded explanation, practical work or responses
to unfamiliar questions. Set review dates when an actual session establishes them.
No fixed completion date, pass probability or mastery percentage is established.
Keep detailed progress in each subject's STATE; update this hub's active subject
and next action, together with the affected subject and session log, after study work.
