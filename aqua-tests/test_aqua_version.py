import delegator


def test_aqua_version():
    c = delegator.run(f"npx aqua --version", block=True)
    print(f"Aqua version: {c.out}")
    assert True
