import pytest


@pytest.fixture
def client() -> str:
    return "client"


@pytest.fixture
def token() -> str:
    return "token"
