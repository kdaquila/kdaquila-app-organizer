import pytest


@pytest.fixture
def user() -> str:
    return "ada"


@pytest.fixture
def secret() -> str:
    return "hunter2"
