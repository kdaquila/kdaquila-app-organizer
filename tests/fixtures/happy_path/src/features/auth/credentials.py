from dataclasses import dataclass


@dataclass
class Credentials:
    user: str
    secret: str
