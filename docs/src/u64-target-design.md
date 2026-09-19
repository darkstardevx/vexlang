# `u64` target design notes

The interpreter supports `u64`, but the first QBE target intentionally rejects it.
Adding `u64` should be a contract change, not an accidental widening of
`vex-scalar-v1`.

## Proposed path

Introduce a new target contract, tentatively `vex-scalar-v2`, once the behavior
below is specified and tested. Keep `vex-scalar-v1` stable for existing QBE
smoke tests.

## Required decisions

| Area | Required decision |
| --- | --- |
| QBE representation | Use QBE `l` for `u64` values and loads/stores with 8-byte slots. |
| Calling convention | Pass and return `u64` as ABI long arguments/results. |
| Arithmetic | Use wrapping-free checked semantics matching the interpreter. |
| Overflow traps | Reuse `101` for checked overflow unless target docs define finer codes. |
| Division traps | Keep division by zero as `102`; unsigned division overflow has no `u64` analogue to `i32::MIN / -1`. |
| Comparisons | Use unsigned QBE comparisons for `<`/`>` and equality for `==`. |
| Conversions | Preserve checked `i32(value)` and `u64(value)` conversion semantics at the target boundary. |
| Mixed integers | Reject implicit mixed-width arithmetic; require existing explicit conversions. |
| Exit behavior | Document that process exit status still truncates to platform status width, so conformance tests should keep expected values small. |

## Implementation checklist

1. Add a second target contract instead of mutating `vex-scalar-v1` silently.
2. Extend target validation to allow `IrType::U64` only under the new contract.
3. Teach QBE emission to select word (`w`) or long (`l`) operations by IR type.
4. Allocate 8-byte slots for `u64` locals and parameters.
5. Add checked `u64` add/sub/mul and division-by-zero traps.
6. Add checked conversion lowering for `i32(...)` and `u64(...)` if conversions
   are allowed through the target subset.
7. Add compiled-vs-interpreted QBE conformance cases for boundaries:
   - `0`, `1`, `u64::MAX` rejection paths where applicable;
   - `u64::MAX - 1 + 1` non-overflow;
   - `u64::MAX + 1` overflow trap;
   - unsigned comparisons above `i32::MAX`;
   - explicit conversion success/failure.

Until these items are complete, `u64` remains intentionally unsupported by the
QBE target even though the interpreter accepts it.
