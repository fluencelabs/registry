import delegator
import random
import json
import ed25519
import os
from config import get_local

def get_sk():
    return ed25519.create_keypair()[0].to_ascii(encoding="base64").decode("utf-8")

def get_relay():
    env = os.environ.get("FLUENCE_ENV")
    if env == "local":
        peers = get_local()
    else:
        if env is None:
            env = "testnet"
        c = delegator.run(f"npx aqua config default_peers {env}", block=True)
        peers = c.out.strip().split("\n")

    assert len(peers) != 0, c.err
    peer = peers[random.randint(0, len(peers) - 1)]
    assert len(peer) != 0, c.err

    return peer

def get_random_peer_id():
    addr = get_relay()
    return addr.split("/")[-1]

def run_aqua(func, args, sk, relay=get_relay()):

    # "a" : arg1, "b" : arg2 .....
    data = {chr(97 + i): arg for (i, arg) in enumerate(args)}
    call = f"{func}(" + ", ".join([chr(97 + i) for i in range(0, len(args))]) + ")"
    file = "./aqua/test.aqua"

    command = f"npx aqua run --addr {relay} -f '{call}' -i {file} --sk {sk} -d '{json.dumps(data)}'"
    print(command)
    c = delegator.run(command, block=True)
    if len(c.err) != 0:
        print(c.err)

    result = json.loads(c.out)
    print(result)
    return result

def create_resource(label, sk):
    result, error = run_aqua("createResource", [label], sk)
    assert len(result) == 1, error
    return result[0]

def get_peer_id(sk):
    return run_aqua("get_peer_id", [], sk)

def test_aqua_version():
    c = delegator.run("npx aqua --version", block=True)
    assert c.out.strip() == "v0.0.1-bug-VM-168-temporary-hack-for-canon-eb5f143-173-1.0"

def test_create_resource():
    sk = get_sk()
    label = "some_label"
    result = create_resource(label, sk)
    peer_id = get_peer_id(sk)
    resource_id = run_aqua("getResourceId", [label, peer_id], sk)
    assert(result == resource_id)

def test_get_resource():
    sk = get_sk()
    label = "some_label"
    resource_id = create_resource(label, sk)
    peer_id = get_peer_id(sk)
    result, error = run_aqua("getResource", [resource_id], sk)
    assert len(result) == 1, error
    resource = result[0]
    assert resource["id"] == resource_id, error
    assert resource["owner_peer_id"] == peer_id, error
    assert resource["label"] == label, error

def test_register_record_unregister():
    sk = get_sk()
    relay = get_relay()
    label = "some_label"
    value = "some_value"
    peer_id = get_peer_id(sk)
    service_id = "id"

    resource_id = create_resource(label, sk)
    result, error = run_aqua("registerServiceRecord", [resource_id, value, peer_id, service_id], sk, relay)
    assert result, error

    # we want at least 1 successful response
    result, error = run_aqua("resolveResource", [resource_id, 1], sk, relay)
    assert len(result) == 1, error

    records = result[0]
    assert len(records) == 1, "records not found"

    record = records[0]
    assert record["metadata"]["key_id"] == resource_id
    assert record["metadata"]["issued_by"] == peer_id
    assert record["metadata"]["peer_id"] == peer_id
    assert record["metadata"]["service_id"] == [service_id]

    result, error = run_aqua("unregisterService", [resource_id, peer_id], sk, relay)
    assert result, error

    result, error = run_aqua("resolveResource", [resource_id, 2], sk, relay)
    assert len(result) == 1, error
    assert len(result[0]) == 0

def test_register_unregister_remote_record():
    sk = get_sk()
    relay = get_relay()
    label = "some_label"
    value = "some_value"
    issuer_peer_id = get_peer_id(sk)
    peer_id = get_random_peer_id()
    service_id = "id"

    resource_id = create_resource(label, sk)
    result, error = run_aqua("registerServiceRecord", [resource_id, value, peer_id, service_id], sk, relay)
    assert result, error

    result, error = run_aqua("resolveResource", [resource_id, 2], sk, relay)
    assert len(result) == 1, error

    records = result[0]
    assert len(records) == 1, "records not found"

    record = records[0]
    assert record["metadata"]["key_id"] == resource_id
    assert record["metadata"]["issued_by"] == issuer_peer_id
    assert record["metadata"]["peer_id"] == peer_id
    assert record["metadata"]["service_id"] == [service_id]

    result, error = run_aqua("unregisterService", [resource_id, peer_id], sk, relay)
    assert result, error

    result, error = run_aqua("resolveResource", [resource_id, 2], sk, relay)
    assert len(result) == 1, error
    assert len(result[0]) == 0
