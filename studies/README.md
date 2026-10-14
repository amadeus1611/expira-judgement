# Persistent study workspace

This folder is the durable record for the owner's study workflow. Read it when
starting or resuming study work, including after a context reset. Repository files
provide continuity when the agent reads them; they do not automatically transfer
chat memory to another tool or environment.

## Resume order

1. [PROFILE.md](PROFILE.md): the owner's stated goals, starting point and preferences.
2. [STATE.md](STATE.md): active goal, teaching subject and next action.
3. [SUBJECTS.md](SUBJECTS.md): stable labels and subject entry points; read the active
   subject's README, STATE and roadmap before teaching it.
4. [PROCEDURES.md](PROCEDURES.md): teaching, code, labs, questions and handoff steps.
5. [ROADMAP.md](ROADMAP.md) and [LABS.md](LABS.md): proposed cross-subject direction and build sequence.
6. [SOURCES.md](SOURCES.md): source identities, inspection status and reuse notes.
7. [SESSION_LOG.md](SESSION_LOG.md): dated work and outcomes; newest entry last.

[HANDOFF.md](HANDOFF.md) is a short entry point for Claude or Codex/GPT taking over.

The hub `STATE.md` points to the active subject's canonical progress and resume point.
Update the affected subject state, hub state and session log at the end of study
work; update the profile only when the owner supplies a change or correction.
Keep confirmed facts, proposals and unresolved items distinct.

## Lesson and practice records

Create material folders under the appropriate subject when actual material exists,
using the layout and stable labels in [SUBJECTS.md](SUBJECTS.md). A shared lesson
has one home, linked by other subjects. Avoid records that imply unperformed work.

- A lesson records its topic, prerequisites, source IDs and relevant sections,
  explanation, lab or exercise, factual claims, and review status.
- A question records its learning/exam objective, stem, assumptions, options,
  answer key, explanation for each option, evidence and validation status.
- A learner result records the question/lesson version, actual response, feedback,
  observed misunderstanding and next review. Leave unobserved scores unset.
- A check records the exact input/source snapshot references, date, command or
  settings, original EXPIRA report, and how flagged claims were resolved.

Save source excerpts or full snapshots only with appropriate reuse rights. Record
the exact inspected section and actual access date. Keep credentials, vouchers and
unrelated personal or client data out of this folder.

## Teaching and checking

Use independent sources to teach, explain principles, demonstrate them, and test
understanding. Source selection and question review accompany EXPIRA's evidence
checks. AI-generated explanations are not independent evidence for other generated
explanations. Free access and open licensing are recorded separately.

Use the existing [ej-study workflow](../crates/ej-study/README.md) for claim checks.
Preserve qualifications when preparing complete claims, preferably one per line.
Retain original verdicts and reasons. Review every flagged or unresolved claim;
`unsure` does not establish that the claim is false.

Before scoring a practice question, review the supported answer key, its uniqueness
under the stated assumptions, and why each distractor is wrong. Distractors are
intentional alternatives, rather than factual assertions to accept as learning
material. Keep ambiguous or unresolved questions out of scored practice. A checked
claim is evidence about the material; mastery requires observed learner performance.

Decision-block tuning remains Claude's responsibility. Study records and question
results are separate from calibration data, frozen benchmarks and model settings.
