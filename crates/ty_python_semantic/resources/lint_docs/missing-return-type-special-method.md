## What it does

Checks for special methods without a return type annotation.

When ty can infer an annotation from the implementation, it offers an unsafe fix. This works even
when return type inference is disabled for calls.

## Why is this bad?

A return annotation documents the result callers can expect and lets ty check that the
implementation returns a compatible value.

## Examples

```python
class Counter:
    def __len__(self):  # error: [missing-return-type-special-method]
        return 0
```

Use instead:

```python
class Counter:
    def __len__(self) -> int:
        return 0
```
