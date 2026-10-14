# FOUND-001 v1 — Information, bits, bytes and programs

Prepared: 2026-10-14 (Asia/Manila). Prerequisites: none. Foundation lesson,
independent of a specific Security+ exam version. First segment only; communication
and packet examples will follow after checking understanding.

Goal: distinguish a bit from a byte, reason about binary patterns and describe
the role of a program. Status: prepared for teaching; learner responses pending.
This record does not establish mastery or completion.

## First teaching segment

A **bit** is one binary digit with two possible values: **0 or 1**. Imagine a switch:
agree that off means 0 and on means 1. This is an analogy for two states, not a
complete description of electronic hardware. **Binary** is also called base-2.
See [CS50, Lecture 0](https://cs50.harvard.edu/x/notes/0/#computer-science-and-problem-solving).

With two independent switches, we can distinguish four patterns. Order matters:
the first and second positions are different positions.

| First bit | Second bit | Pattern |
|---|---|---|
| 0 | 0 | `00` |
| 0 | 1 | `01` |
| 1 | 0 | `10` |
| 1 | 1 | `11` |

Each extra bit doubles the possibilities: every existing pattern can gain either
a 0 or a 1. Thus three bits give 2 × 2 × 2 = 8 patterns. These are mathematical
counts of possible patterns, not measurements of an actual device.

A **byte** contains **eight bits**. Therefore eight independent bits give
2 × 2 × 2 × 2 × 2 × 2 × 2 × 2 = 256 possible patterns. A byte is a unit of data;
it is not synonymous with a whole file, number or character.
See [CS50, Lecture 0](https://cs50.harvard.edu/x/notes/0/#computer-science-and-problem-solving).

A **program** specifies instructions for performing a computation. For an original
toy example, instructions could say: take two input numbers, add them, and display
the result. With inputs 2 and 3, that gives output 5. The numbers are the data;
the instructions say what to do with them. See [Think Python, section 1.1](https://greenteapress.com/thinkpython2/html/thinkpython2002.html).

## Unscored check of understanding

Answer in your own words; these are foundation exercises, not official exam items.

1. `Q-FOUND-001-01`: For three independent bits, with both values allowed in each
   position, how many patterns are possible? List them.
2. `Q-FOUND-001-02`: How many bits are in two bytes? Explain the conversion.
3. `Q-FOUND-001-03`: In the toy addition example, identify the input data,
   the instruction that changes it, and the output.

Tutor reference: (1) eight: 000, 001, 010, 011, 100, 101, 110, 111;
(2) 2 × 8 = 16 bits; (3) inputs 2 and 3, add them, output 5. Reasoning is checked
by enumeration/arithmetic and the program definition. There are no distractors.
No learner answers or scores are recorded yet.

## Evidence and review

Source identities/excerpts: [sources/README.md](sources/README.md).
Claim files: [CS50 claims](cs50-claims.txt), [program claim](program-claims.txt).
Original engine findings, actual arithmetic execution and review: [REVIEW.md](REVIEW.md).
The three definition paraphrases remain engine-`unsure`, manually reviewed against
the original source. Do not describe this lesson as fully EXPIRA-verified.

The explanations, analogy, table and exercises are original teaching material.
The CS50-derived explanation is offered under [CC BY-NC-SA 4.0](https://creativecommons.org/licenses/by-nc-sa/4.0/),
with attribution above and adaptations identified here. Short unchanged source
quotations are separately attributed in `sources/README.md`; no endorsement implied.
