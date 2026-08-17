def authenticate(user: str) -> bool:
    return validate_token(user)


def validate_token(token: str) -> bool:
    return bool(token)
