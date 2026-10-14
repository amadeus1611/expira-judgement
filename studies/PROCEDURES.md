# Teaching, lab and handoff procedures

Use these procedures for study work with either Codex/GPT or Claude. They describe
the workflow; they are not a record that any lesson, lab or assessment has occurred.

## Resume a session

1. Read repository instructions and `studies/AGENTS.md`, then the resume files in
   `studies/README.md`. Read the latest log entry and relevant lesson/lab records.
   Identify the active subject in `SUBJECTS.md`; read its own STATE and roadmap.
2. Identify the last demonstrated skill, unresolved misconception and next action.
   Use actual learner responses, not assumed knowledge from earlier explanations.
3. Recheck source versions or repository/tool status only when relevant to the work.
4. Continue from that point. Ask only for missing information needed by the next task.

## Switch or add subjects

1. Use the requested subject's registry ID and canonical state. For a new subject,
   record its stated goal, prerequisites and curriculum/syllabus when supplied.
2. Link shared prerequisites instead of duplicating their progress or assuming
   mastery from another track. Keep lessons, questions, labs and results labelled
   with subject and material version.
3. Match validation to the discipline: source/EXPIRA checks for factual claims;
   derivations, units and independent calculations for mathematics; execution and
   appropriate tests for code; actual observations for experiments. Identify
   interpretations and uncertainty explicitly. A source-support verdict alone does
   not validate a calculation or prove an experimental result.
4. Update the hub's active goal/subject and the affected subject's next action.

## Teach a concept and its code

1. State the goal and prerequisites. Explain unfamiliar terms before using them.
2. Inspect an independent source and cite the relevant passage/version. Prepare
   complete factual claims and review EXPIRA flags before relying on the material.
3. Explain the mechanism with a small example. For code, show a minimal working
   piece, explain each new construct, and ask the learner to predict its behaviour.
4. Run it in the appropriate environment. Compare actual output with the prediction,
   then change one thing so the learner sees how the result changes.
5. Teach debugging from actual errors: reproduction, observation, hypothesis,
   a small change and verification. Record what was observed rather than invented output.
6. Ask for an explanation and a related task with different inputs or conditions.
   Record hints, responses and remaining gaps; demonstrated understanding needs evidence.

## Build a lab when its prerequisites matter

1. Select a lab from `LABS.md` and confirm the learner's platform and prerequisites.
2. Define its learning goal, owned/authorized targets, isolation, required tools,
   expected observations, completion criteria and reset/cleanup procedure.
3. Build teaching artifacts outside the decision crates, using small tested changes.
   Version the environment and code; distinguish expected results from actual results.
4. Walk through setup, code/configuration, execution, interpretation and debugging.
   The learner should be able to explain and repeat the relevant steps.
5. Save the lab version, commands, outputs, observed results, review status and next task.
   Reproduce its reset procedure before describing it as repeatable.

## Write and use a practice question

1. Record the objective and prerequisites. Use confirmed official exam objectives
   for exam mapping; foundation diagnostics can be marked as independent of an exam version.
2. Save the stem, assumptions, options, answer key and reasoning for every option.
3. Link the answer and explanation to independent evidence. Check factual claims with
   EXPIRA and review meaning, assumptions, unique answer and distractor reasoning.
4. Keep unresolved questions in draft/unscored status. Distinguish a learner's wrong
   answer from a flawed question; correct and version the question when needed.
5. Grade only reviewed questions. Record the actual response, question version,
   explanation given, hints and review action. Avoid presenting practice scores as
   measured probabilities of passing the real exam.

## End a session or hand off to another agent

- Owner preference (2026-10-14): save substantive study exchanges promptly in Git,
  including actual learner responses and the presented lesson version. Summarize
  the relevant exchange faithfully; do not imply these files capture every UI event.
  Commit one coherent tested change, run the required Cargo checks before pushing,
  and merge the corresponding PR after applicable validation passes. This is standing
  authorization to merge this owner's study-work changes, not unrelated agent work.
  Do not bypass required reviews/checks or weaken CI to speed up saving.
- GitHub server auto-merge was disabled when inspected on 2026-10-14; enabling it
  was denied HTTP 403. `gh pr merge --auto` can merge immediately when no required
  gate prevents it, so it is not proof of a pending-check queue. Inspect the exact
  head and validation before merging; preserve unmerged work on its pushed branch
  if a required gate or permissions prevent merging. No background watcher is installed.
- Save the material and relevant source/report references. Preserve existing results
  with their versions when explanations or questions are corrected.
- Update the affected subject's STATE: observed progress, unresolved issues and next action.
  Update the hub `STATE.md` with the active goal/subject and resume links.
- Append `SESSION_LOG.md`: work performed, responses/results, open items and next action.
- Keep `HANDOFF.md` in this folder concise and accurate for the next teacher.
- A coding session also appends the repository's root `HANDOFF.md` as its rules require.
- Never substitute chat history, self-generated answers or memory of a source for an
  actual source record. Do not write credentials or infer missing learner results.
