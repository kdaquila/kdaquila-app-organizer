def authenticate(user: str) -> bool:
    if not user:
        return False
    if user.startswith("_"):
        return False
    if len(user) > 64:
        return False
    if " " in user:
        return False
    return True
