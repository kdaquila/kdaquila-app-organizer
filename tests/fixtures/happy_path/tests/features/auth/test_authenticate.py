from src.features.auth.authenticate import authenticate


def test_accepts_a_user() -> None:
    assert authenticate("ada")


def test_rejects_an_empty_user() -> None:
    assert not authenticate("")
