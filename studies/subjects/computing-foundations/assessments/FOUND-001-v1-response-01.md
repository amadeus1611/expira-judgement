# FOUND-001 v1 — First diagnostic responses

Subject: `FOUND`. Date: 2026-10-14 (Asia/Manila).
Material: [FOUND-001 v1](../lessons/FOUND-001-v1/README.md).
Assessment: unscored foundation diagnostic; no exam-readiness inference.

## Learner's initial response

Original submitted wording and pattern list, with extra blank lines omitted:

```text
1. 3 bits can form about 6 combinations.
001
010
100
011
101
110
2. About 16 bits, 2x8
3. The input is the two numbers 2 and 3, the instructions are what to do with them, the sum, and the output is the result.
```

## Feedback and observed evidence

- `Q-FOUND-001-01`: six distinct valid three-bit patterns were listed. The complete
  set contains eight; `000` and `111` were missing. All-off and all-on patterns are
  permitted under the question's assumptions. Each of three independent positions
  has two choices, so the count is 2 × 2 × 2 = 2^3 = 8. Do not infer how the learner
  arrived at six, or that they deliberately excluded those patterns.
- `Q-FOUND-001-02`: correct conversion and calculation, 2 × 8 = 16 bits.
  The count is exact. This demonstrates the given conversion example;
  broader unit-conversion skill is not yet assessed.
- `Q-FOUND-001-03`: correctly identifies the two numbers as input, the operation as
  instructions and its result as output. The operation is addition; the numerical
  output is 5. Their response gives the roles without writing 5.

The tutor explained the two omitted patterns and multiplication of independent
choices. Mathematical feedback was checked against the saved [enumeration output](../lessons/FOUND-001-v1/reports/arithmetic-demo.json)
and original lesson reference. Source reports/verdicts were not changed or rerun
to grade this arithmetic exercise.

## Learner's follow-up after correction

```text
Oh okay i forgot that it is 2^3 my bad.
```

Observed: the learner recognizes the correct expression after feedback. This is
correction acknowledgement, not an independent transfer answer or whole-lesson
mastery. The tutor clarified that 2 counts choices per bit and the exponent counts
bit positions.

## Next exercise — Q-FOUND-001-04

Prompt: With four independent bits, and both values allowed in each position, how
many patterns can you form? Explain what the base and exponent represent.

Tutor reference: 2^4 = 16 patterns; two choices in each of four positions.
This is an unscored transfer check; response and completion are pending. No hints
beyond the preceding rule explanation have been recorded.
