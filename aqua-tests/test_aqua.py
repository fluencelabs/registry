import delegator
import random
import json
import os
from config import get_local
import inspect


def get_function_name():
    return inspect.stack()[1][3]


def get_relay():
    env = os.environ.get("FLUENCE_ENV")
    if env == "local":
        peers = get_local()
    else:
        if env is None:
            env = "testnet"
        c = delegator.run(f"node ./getDefaultPeers.js -- {env}", block=True)
        peers = c.out.strip().split("\n")

    assert len(peers) != 0, c.err
    peer = peers[random.randint(0, len(peers) - 1)]
    assert len(peer) != 0, c.err

    return peer


def get_random_peer_id():
    addr = get_relay()
    return addr.split("/")[-1]


def run_aqua(func, args, k, relay=get_relay()):

    # "a" : arg1, "b" : arg2 .....
    data = {chr(97 + i): arg for (i, arg) in enumerate(args)}
    call = f"{func}(" + ", ".join([chr(97 + i)
                                   for i in range(0, len(args))]) + ")"

    command = f"npx fluence run --relay {relay} -f '{call}' -k '{k}' --data '{json.dumps(data)}' --import 'node_modules' --quiet"
    print(command)
    c = delegator.run(command, block=True)
    if len(c.err) != 0:
        print(c.err)

    try:
        result = json.loads(c.out)
        print(result)
        return result
    except:
        print(c.out)
        return c.out


def create_resource(label, k):
    result, error = run_aqua("createResource", [label], k)
    assert result != None, error
    return result


def get_peer_id(k):
    return run_aqua("get_peer_id", [], k)


def test_create_resource():
    k = get_function_name()
    label = "some_label"
    result = create_resource(label, k)
    peer_id = get_peer_id(k)
    resource_id = run_aqua("getResourceId", [label, peer_id], k)
    assert result == resource_id


def test_get_resource():
    k = get_function_name()
    label = "some_label"
    resource_id = create_resource(label, k)
    peer_id = get_peer_id(k)
    result, error = run_aqua("getResource", [resource_id], k)
    assert result != None, error
    assert result["id"] == resource_id, error
    assert result["owner_peer_id"] == peer_id, error
    assert result["label"] == label, error


def test_register_record_unregister():
    k = get_function_name()
    relay = get_relay()
    label = "some_label"
    value = "some_value"
    peer_id = get_peer_id(k)
    service_id = "id"

    resource_id = create_resource(label, k)
    result, error = run_aqua(
        "registerService", [resource_id, value, peer_id, service_id], k, relay)
    assert result, error

    # we want at least 1 successful response
    result, error = run_aqua("resolveResource", [resource_id, 1], k, relay)
    assert result != None, error

    assert len(result) == 1, "records not found"

    record = result[0]
    assert record["metadata"]["key_id"] == resource_id
    assert record["metadata"]["issued_by"] == peer_id
    assert record["metadata"]["peer_id"] == peer_id
    assert record["metadata"]["service_id"] == [service_id]

    result, error = run_aqua("unregisterService", [resource_id, peer_id],
                             k, relay)
    assert result, error

    result, error = run_aqua("resolveResource", [resource_id, 2], k, relay)
    assert result != None, error
    assert len(result) == 0


def test_register_unregister_remote_record():
    k = get_function_name()
    relay = get_relay()
    label = "some_label"
    value = "some_value"
    issuer_peer_id = get_peer_id(k)
    peer_id = get_random_peer_id()
    service_id = "id"

    resource_id = create_resource(label, k)
    result, error = run_aqua(
        "registerService", [resource_id, value, peer_id, service_id], k, relay)
    assert result, error

    result, error = run_aqua("resolveResource", [resource_id, 2], k, relay)
    assert result != None, error

    assert len(result) == 1, "records not found"

    record = result[0]
    assert record["metadata"]["key_id"] == resource_id
    assert record["metadata"]["issued_by"] == issuer_peer_id
    assert record["metadata"]["peer_id"] == peer_id
    assert record["metadata"]["service_id"] == [service_id]

    result, error = run_aqua("unregisterService", [resource_id, peer_id],
                             k, relay)
    assert result, error

    result, error = run_aqua("resolveResource", [resource_id, 2], k, relay)
    assert result != None, error
    assert len(result) == 0
