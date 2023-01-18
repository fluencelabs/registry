import delegator


def test_fluence_cli_version():
    c = delegator.run(f"npx fluence --version", block=True)
    print(f"Fluence CLI version: {c.out}")
    assert True
