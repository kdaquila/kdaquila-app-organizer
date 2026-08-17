def authenticate(user: str) -> bool:
    return _check(user)


def _check(user: str) -> bool:
    return bool(user)
