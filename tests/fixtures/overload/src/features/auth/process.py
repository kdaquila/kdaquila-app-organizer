from typing import overload


@overload
def process(value: int) -> int: ...
@overload
def process(value: str) -> str: ...
def process(value):
    return value
