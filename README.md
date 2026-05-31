# lau-kintsugi

Break things. Repair them with gold. The cracks become the most beautiful part.

Inspired by the Japanese art of kintsugi — repairing broken pottery with gold-dusted lacquer — this crate treats failures as first-class data. When a build fails, an agent crashes, or a room dissolves, the repair is visible and MORE valuable than the original. The gold in the cracks is the learning.

## The concept in 60 seconds

Most systems hide their failures. Kintsugi makes them visible and valuable:

- **Breaks** are recorded, not just logged — every failure gets an ID, a type, and a context
- **Repairs** are explicit — you describe what you did to fix it
- **Golden repairs** are the ones that made the system *better* than before
- **Resilience scoring** tracks how the system improves through break/repair cycles
- **Lessons learned** are extracted from repair patterns

The philosophy: a system that has broken and been repaired is stronger than one that has never broken. The gold isn't decorative — it's structural.

## Quick start

```rust
use lau_kintsugi::{KintsugiLedger, Break, Repair, BreakType};

let mut ledger = KintsugiLedger::new();

// Record a break
let break_id = ledger.record_break(Break {
    break_type: BreakType::BuildFailure,
    context: "lau-math-42: test overflow in geodesic computation".into(),
    severity: 0.7,
});

// Record the repair — what you actually did
ledger.record_repair(&break_id, Repair {
    description: "Switched to RK4 integrator with adaptive step size".into(),
    golden: true, // this repair made the system better
    improvement: 0.3,
});

// Check resilience score
let score = ledger.resilience_score(); // goes up with each golden repair

// Get lessons learned
for lesson in ledger.lessons() {
    println!("{}: {}", lesson.break_type, lesson.pattern);
}
```

## Key types

| Type | What it is |
|------|-----------|
| `KintsugiLedger` | The golden repair book — records all breaks and repairs |
| `Break` | A recorded failure with type, context, and severity |
| `Repair` | A fix applied to a break — golden if it improved the system |
| `ResilienceScore` | How much stronger the system has become through repairs |
| `Lesson` | A pattern extracted from repair history |

## Contributing

[Open an issue](https://github.com/SuperInstance/lau-kintsugi/issues) or PR.
