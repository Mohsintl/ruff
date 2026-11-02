# Missing return annotations

```toml
[rules]
missing-return-type-public-function = "error"
missing-return-type-private-function = "error"
missing-return-type-special-method = "error"
missing-return-type-static-method = "error"
missing-return-type-class-method = "error"
```

## Functions

The fix uses the types of reachable return expressions. Enabling the rule alone leaves the return
type seen by callers as `Unknown`.

```py
def increment(value: int):  # snapshot: missing-return-type-public-function
    return value + 1

reveal_type(increment(1))  # revealed: Unknown

def _helper():  # snapshot: missing-return-type-private-function
    return "ready"

def annotated() -> str:  # ok
    return "ready"
```

```snapshot
error[missing-return-type-public-function]: Missing return type annotation for public function `increment`
 --> src/mdtest_snippet.py:1:5
  |
1 | def increment(value: int):  # snapshot: missing-return-type-public-function
  |     ^^^^^^^^^
help: Add return type annotation `int`
  |
  - def increment(value: int):  # snapshot: missing-return-type-public-function
1 + def increment(value: int) -> int:  # snapshot: missing-return-type-public-function
2 |     return value + 1
  |
note: This is an unsafe fix and may change runtime behavior


error[missing-return-type-private-function]: Missing return type annotation for private function `_helper`
 --> src/mdtest_snippet.py:6:5
  |
6 | def _helper():  # snapshot: missing-return-type-private-function
  |     ^^^^^^^
help: Add return type annotation `str`
  |
5 |
  - def _helper():  # snapshot: missing-return-type-private-function
6 + def _helper() -> str:  # snapshot: missing-return-type-private-function
7 |     return "ready"
  |
note: This is an unsafe fix and may change runtime behavior
```

## Methods

Class and static methods have their own rules, even when their names are private.

```py
class Counter:
    def __init__(self):  # snapshot: missing-return-type-special-method
        self.value = 0

    @staticmethod
    def _parse(text: str):  # snapshot: missing-return-type-static-method
        return int(text)

    @classmethod
    def _unit(cls):  # snapshot: missing-return-type-class-method
        return 1
```

```snapshot
error[missing-return-type-special-method]: Missing return type annotation for special method `__init__`
 --> src/mdtest_snippet.py:2:9
  |
2 |     def __init__(self):  # snapshot: missing-return-type-special-method
  |         ^^^^^^^^
help: Add return type annotation `None`
  |
1 | class Counter:
  -     def __init__(self):  # snapshot: missing-return-type-special-method
2 +     def __init__(self) -> None:  # snapshot: missing-return-type-special-method
3 |         self.value = 0
  |
note: This is an unsafe fix and may change runtime behavior


error[missing-return-type-static-method]: Missing return type annotation for static method `_parse`
 --> src/mdtest_snippet.py:6:9
  |
6 |     def _parse(text: str):  # snapshot: missing-return-type-static-method
  |         ^^^^^^
help: Add return type annotation `int`
  |
5 |     @staticmethod
  -     def _parse(text: str):  # snapshot: missing-return-type-static-method
6 +     def _parse(text: str) -> int:  # snapshot: missing-return-type-static-method
7 |         return int(text)
  |
note: This is an unsafe fix and may change runtime behavior


error[missing-return-type-class-method]: Missing return type annotation for class method `_unit`
  --> src/mdtest_snippet.py:10:9
   |
10 |     def _unit(cls):  # snapshot: missing-return-type-class-method
   |         ^^^^^
help: Add return type annotation `int`
   |
9  |     @classmethod
   -     def _unit(cls):  # snapshot: missing-return-type-class-method
10 +     def _unit(cls) -> int:  # snapshot: missing-return-type-class-method
11 |         return 1
   |
note: This is an unsafe fix and may change runtime behavior
```

## Implicit class and static methods

```py
class Base:
    def __new__(cls): ...  # error: [missing-return-type-static-method]
    def __init_subclass__(cls): ...  # error: [missing-return-type-class-method]
```

## Visibility

A nested function, a member of a private class, a property setter, and a function omitted from
`__all__` are all private.

```py
__all__ = ["Public"]

class Public:
    @property
    def size(self):  # error: [missing-return-type-public-function]
        return 1

    @size.setter
    def size(self, value: int): ...  # error: [missing-return-type-private-function]

class _Private:
    def method(self): ...  # error: [missing-return-type-private-function]

def helper():  # error: [missing-return-type-private-function]
    def nested(): ...  # error: [missing-return-type-private-function]
```

## Private modules

`_internal/api.py`:

```py
def run(): ...  # error: [missing-return-type-private-function]
```

## Overloads

Only the overload signatures require an annotation. A stub body gives ty no return type to suggest.

```py
from typing import overload

@overload
def parse(value: int): ...  # snapshot: missing-return-type-public-function
@overload
def parse(value: str) -> str: ...
def parse(value):  # ok
    return value
```

```snapshot
error[missing-return-type-public-function]: Missing return type annotation for public function `parse`
 --> src/mdtest_snippet.py:4:5
  |
4 | def parse(value: int): ...  # snapshot: missing-return-type-public-function
  |     ^^^^^
```

## Imported return type

`models.py`:

```py
class Item: ...

def load() -> Item:
    return Item()
```

`client.py`:

```py
from models import load

def item():  # snapshot: missing-return-type-public-function
    return load()
```

```snapshot
error[missing-return-type-public-function]: Missing return type annotation for public function `item`
 --> src/client.py:3:5
  |
3 | def item():  # snapshot: missing-return-type-public-function
  |     ^^^^
help: Add return type annotation `Item`
  |
  - from models import load
1 + from models import load, Item
2 |
  - def item():  # snapshot: missing-return-type-public-function
3 + def item() -> Item:  # snapshot: missing-return-type-public-function
4 |     return load()
  |
note: This is an unsafe fix and may change runtime behavior
```

## Annotations on older Python versions

On Python 3.9, an inferred union needs to be quoted so the annotation can be evaluated at runtime.

```toml
[environment]
python-version = "3.9"

[rules]
missing-return-type-public-function = "error"
```

```py
str = 0

def label(active: bool):  # snapshot: missing-return-type-public-function
    if active:
        return "active"
```

```snapshot
error[missing-return-type-public-function]: Missing return type annotation for public function `label`
 --> src/mdtest_snippet.py:3:5
  |
3 | def label(active: bool):  # snapshot: missing-return-type-public-function
  |     ^^^^^
help: Add return type annotation `'builtins.str | None'`
  |
1 + import builtins
2 | str = 0
3 |
  - def label(active: bool):  # snapshot: missing-return-type-public-function
4 + def label(active: bool) -> 'builtins.str | None':  # snapshot: missing-return-type-public-function
5 |     if active:
  |
note: This is an unsafe fix and may change runtime behavior
```

## Incompatible inherited return type

When an override returns a value incompatible with its inherited annotation, ty leaves the choice of
annotation to the author.

```py
class Base:
    def value(self) -> str:
        return "value"

class Child(Base):
    def value(self):  # snapshot: missing-return-type-public-function
        return 1
```

```snapshot
error[missing-return-type-public-function]: Missing return type annotation for public function `value`
 --> src/mdtest_snippet.py:6:9
  |
6 |     def value(self):  # snapshot: missing-return-type-public-function
  |         ^^^^^
```

## Conditional type definitions

An annotation evaluated at function definition time can refer to a class only available on some
paths. In that case ty reports the missing annotation without suggesting the class name.

```py
from random import random

if random() < 0.5:
    class Item: ...

def load():  # snapshot: missing-return-type-public-function
    return Item()
```

```snapshot
error[missing-return-type-public-function]: Missing return type annotation for public function `load`
 --> src/mdtest_snippet.py:6:5
  |
6 | def load():  # snapshot: missing-return-type-public-function
  |     ^^^^
```
