# Subject registry and labels

Use stable subject IDs so lessons, progress and assessments remain distinct when
switching subjects. Shared prerequisites have one canonical record and are linked
from each track that uses them.

| ID | Subject | Resume entry | Current role |
|---|---|---|---|
| `FOUND` | Computing foundations | [Computing foundations](subjects/computing-foundations/README.md) | `FOUND-001` in progress; first diagnostic reviewed, transfer response pending |
| `SECPLUS` | CompTIA Security+ preparation | [Security+](subjects/security-plus/README.md) | Primary near-term goal; prerequisites first, exam version unconfirmed |
| `CHEM` | General chemistry (OpenStax) | [Chemistry](subjects/chem/README.md) | Opened 2026-10-20; book choice and placement diagnostic pending |
| `EE` | Electrical engineering | [Electrical engineering](subjects/electrical-engineering/README.md) | Intended future degree; specific courses not selected |
| `BIO` | Light general-biology review (optional) | [Biology](subjects/bio/README.md) | Opened 2026-10-20; awaiting book sections; close if no science part is confirmed |

Manufacturing, semiconductors and industrial/embedded security are recorded interests,
not assumed enrollments or completed courses. Add other subjects when requested;
record their actual goals, prerequisites and curriculum sources.

## Record layout

- `subjects/<subject-slug>/README.md`: subject identity and entry point.
- `subjects/<subject-slug>/STATE.md`: canonical progress, gaps and next action.
- `subjects/<subject-slug>/ROADMAP.md`: proposed sequence when a plan exists.
- Actual material goes under that subject's `lessons/`, `questions/`, `labs/`,
  `assessments/` and `reports/` as needed. Do not create completed-looking placeholders.

Label lessons `<SUBJECT>-001`, questions `Q-<SUBJECT>-001-01`, and labs
`LAB-<SUBJECT>-001`. Include subject ID, material version and actual date in assessment
and report records. Source IDs remain global in [SOURCES.md](SOURCES.md), with subject tags.

For example, a computing lesson used by both Security+ and EE remains in `FOUND`;
the other tracks link to its evidence and results. Do not copy its progress into
multiple states or infer mastery in another subject from its completion.
