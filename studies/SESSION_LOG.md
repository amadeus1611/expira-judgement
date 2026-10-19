# Study session log

Append dated entries. Record actual work and outcomes, open items and next action.
Use Asia/Manila for user-facing dates. Keep assessment results tied to material versions.

## 2026-10-14 — Orientation and persistence setup

### Recorded
- Owner reports starting from scratch and being available to study full time.
- Goal: source-grounded Security+ preparation, followed by intended EE study and
  longer-term defensive cybersecurity work relevant to manufacturing and the Philippines.
- Assistant role clarified: material guide and tutor, with reviewed practice and
  pre-exam preparation supported by the existing EXPIRA pipeline.
- Initial MIT, CNP3 and NIST page references inspected in the planning conversation.
  Official live CompTIA pages could not be retrieved; exam version is not confirmed.
- Owner authorized a separate persistent study folder. Created the resume files
  and linked them from repository agent instructions.
- Owner also requested explicit code instruction, sandbox labs at the appropriate
  stages, a later Claude/GPT-assisted vulnerability/exploit-checking system, and
  procedural memory for Claude takeover. Added teaching/lab procedures, a proposed
  lab/system progression and a concise handoff. Production readiness remains unestablished.
- Owner requested labelled records for other subjects and a personal school/tutor/guide
  using open materials. Added the `FOUND`, `SECPLUS` and `EE` registry, separate subject
  states/plans and subject-switching procedures; shared prerequisites have one home.
- Existing integration PR #12 is an open draft with reported CI checks successful,
  for integration commit `a21c196`, as inspected today. No lesson, learner assessment
  or exam question bank is completed.
- Memory documentation checks: local Markdown links resolve across 19 study documents;
  formatting, workspace clippy and locked workspace tests pass (135 passed, zero failed,
  one opt-in real-engine test ignored here and previously run explicitly). No source,
  configuration or existing test files changed in this documentation update.

### Open
- Exam version/date, actual daily schedule, equipment and lab setup.
- Source selections and evidence checks for the first lesson; learner baseline assessment.
- Actual lab/platform setup and later system architecture/release criteria.

### Next
Prepare and teach `FOUND-001`, save its evidence and review status, and record the
learner's actual responses before advancing. Resume through `README.md` and `STATE.md`.

## 2026-10-14 — FOUND-001 first segment and saving preference

### Recorded
- Owner requested beginning the first lesson, prompt/output saving and automatic
  merging, then asked to use proper conjecture so learning can start. Recorded
  standing merge authorization and explicit assumptions/analogy labelling.
- GitHub auto-merge setting was disabled; changing it and reading branch protection
  were denied HTTP 403. Marked PR #12 ready and `gh pr merge --auto` merged it immediately
  at head `944c64e`, with check jobs and one full mutation job successful; a duplicate
  mutation job was still running. Future merges must inspect applicable validation
  explicitly instead of relying on `--auto` to wait. No background watcher installed.
- Prepared `FOUND-001` v1, with independently inspected CS50 and Think Python
  excerpts. Saved exact pipeline commands/reports: three paraphrases `unsure`
  (no judge available), one quotation `backed`. Reviewed flags without changing them.
- Ran tutor-side Python enumeration/arithmetic and saved actual output. Prepared
  and presented the bit/byte segment and three unscored exercises. No learner answers,
  grades or mastery inferred from agreement to start.
- Saving checks pass: formatting, workspace clippy, locked workspace tests
  (135 passed, zero failed), local links and report/evidence consistency. All
  code, existing tests, configuration, dependencies and CI remain unchanged.

### Open
- Diagnostic responses, learner platform constraints and official exam version/date.
- No local model/policy installed; manually reviewed facts retain unresolved engine findings.

### Next
Record actual answers to `Q-FOUND-001-01` through `03`, explain gaps and continue
the lesson. Save subsequent substantive exchanges through `PROCEDURES.md`.

## 2026-10-14 — FOUND-001 diagnostic feedback

### Recorded
- Saved the actual initial answers against lesson v1: six valid three-bit patterns
  with `000`/`111` missing; correct 16-bit conversion for two bytes; correct roles
  for input, instructions and output in the addition example.
- Explained eight patterns and independent choices; learner replied that they
  forgot `2^3`. Recorded that acknowledgement without inferring independent mastery.
- Prepared unscored `Q-FOUND-001-04` to apply the rule to four independent bits.
  Its response is pending. Preserved lesson v1 and original source/verdict files.
- Previous lesson PR #13 is merged; all four reported CI jobs were successful when
  inspected this session. Updated the subject/hub state and teacher resume point.

### Open
- Independent application of the counting rule, platform constraints and exam version/date.

### Next
Review the four-bit response before advancing or assigning further counting practice.

## 2026-10-16 — GREEK added; ch. 16 and present infinitive `-ειν`

### Recorded
- Added subject `GREEK` (Mounce BBG 2nd ed.) with README, STATE, ROADMAP and a
  chapter-by-chapter TRACKER. Owner works on Track Two (verbs early).
- Per the owner, lessons completed in Codex are assumed done: ch. 1–9 and 15.
  Codex "lesson 4c" mapped to ch. 16 Present Active Indicative (owner confirmed).
- ch. 16: 10 claims checked with ej-study `--lines`: 8 backed; 1 paraphrase unsure
  (restated in the book's wording); 1 deliberate false test claim (ετε as first
  plural) came back `unsure`, not `mismatch`, and was dropped.
- `-ειν` (from ch. 32): 11 claims checked: 10 backed; 1 deliberate false test claim
  unsure and dropped. Reports in `subjects/greek/reports/`.
- No local judge model or policy installed in this session.

- Re-taught `-ειν` in mentor style at the owner's request. Lesson test (10 questions),
  observed answers: 9/10. Ch. 16 and `-ειν` now `demonstrated`; missed the
  connecting-vowel rule (ο before μ/ν, else ε), marked `needs review`.

### Open
- Connecting-vowel rule: restated correctly after correction, not yet recalled cold.

### Next
Recall the connecting-vowel rule, then ch. 17 Contract Verbs.

## 2026-10-18 — GREEK ch. 17 Contract Verbs; review plateau planned

### Recorded
- Connecting-vowel rule recalled cold (ε before τε) → ch. 16 rule `demonstrated`.
- Ch. 17 taught from BBG §17.1–17.8: contract vowels, Big Five, rules 6–8, paradigm,
  elimination method. 11 claims checked with ej-study `--lines`: 1 backed, 10 unsure
  (no judge installed; the OCR source splits words, so exact matching misses); all
  checked by hand against the book. Report in `subjects/greek/reports/`.
- Observed: contractions rebuilt and ποιοῦσι parsed correctly; elimination method used
  on πληροῦτε. Ch. 17 `demonstrated`. Slips: ρ as "p", rule numbers left out.
- Typing convention: `w` = ω, `h` = η.
- Owner chose a review plateau over ch. 1–17 as studied; plan in `subjects/greek/REVIEW.md`.

### Open
- ej-study can't match word for word against the OCR book text (split words); a cleaned
  source excerpt or a local judge would let claims come back `backed`.

### Next
Greek review block A: alphabet and the ρ/ν/ω/η look-alike drill.
- Later the same session: switched GREEK to Track One. The owner had followed it in
  their own study, and the verbs-first order was confusing; Mounce also prefers it
  (p. 74). Order now: review ch. 1–9, review the verb blocks, then ch. 10–14, then 18 on.
- Built an interactive alphabet typing drill (`subjects/greek/drill/greek-drill.html`,
  private artifact). Owner's pasted results: 87/87 first try across five decks. Block A
  alphabet part, pass 1. Next: §3.4–3.5 and ch. 4.

## 2026-10-18 (Greek review, continued)

### Done
- Block A pass 1 complete: ch. 3 §3.4–3.5 and ch. 4 decks added; 146/146 first try overall.
- Block B pass 1: ch. 6 decks (article, parse, noun rules, endings, decline, read the article,
  subject/object) and a ch. 6 study sheet on the drill page. 66/66 on forms; two word-order
  slips on subject/object, each fixed on re-run.
- Fixes from owner feedback: γγ said "ng" in the split deck (a pan gel lo men); the Greek split
  and rule now show after every syllable answer.
- Owner asked that all quizzes run in the artifact rather than chat.
- Drill artifact moved to a new URL (see REVIEW.md).

### Open
- Word order: the owner still reads position before ending in object-first sentences.

### Next
Block A pass 2 and block B pass 2 (cold), then block C (ch. 7 genitive and dative).

## 2026-10-19 — Greek review, block C pass 1

### Done
- Drill page rebuilt (new layout, study sheets, full ch. 7 decks); ch. 7 rules deck fixed (ἁμαρτία had no choices) and rewritten as plain questions with their own choices.
- Owner ran every deck: A–B clean (one typo, ι typed "Iot", re-run 24/24); block C pass 1 151/151 first try, Mixed 20/20.

### Open
- Word order in object-first sentences: clean this session (13/13), keep watching.

### Next
Block C pass 2 (cold, separate session), then block D (ch. 8 prepositions and εἰμί).

