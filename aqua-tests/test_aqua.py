import delegator
import random
import json
import os
import inspect
from config import get_local

delegator.run("npx fluence dep npm i", block=True)
default_peers = json.loads(delegator.run(
    f"node ./getDefaultPeers.js", block=True).out)


def get_relays():
    env = os.environ.get("FLUENCE_ENV")
    if env == "local":
        peers = get_local()
    else:
        if env is None:
            env = "testnet"
        peers = [peer["multiaddr"] for peer in default_peers[env]]

    assert len(peers) != 0
    return peers


relays = get_relays()
peer_ids = [relay.split("/")[-1] for relay in relays]


def get_random_list_item(ar):
    return ar[random.randint(0, len(ar) - 1)]


def get_random_relay():
    return get_random_list_item(relays)


def get_random_peer_id():
    return get_random_list_item(peer_ids)


def get_label():
    return ''.join(random.choice('0123456789ABCDEF') for i in range(16))


def run_aqua(func, args, relay=get_random_relay()):

    # "a" : arg1, "b" : arg2 .....
    data = {chr(97 + i): arg for (i, arg) in enumerate(args)}
    call = f"{func}(" + ", ".join([chr(97 + i)
                                   for i in range(0, len(args))]) + ")"

    test_name = inspect.stack()[-34][3]
    command = f"npx fluence run -k {test_name} --relay {relay} -f '{call}' --data '{json.dumps(data)}' --import 'node_modules' --quiet"
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


def create_resource(label):
    result, error = run_aqua("createResource", [label])
    assert result != None, error
    return result


def get_peer_id():
    return run_aqua("get_peer_id", [])


def test_create_resource():
    label = get_label()
    result = create_resource(label)
    peer_id = get_peer_id()
    resource_id = run_aqua("getResourceId", [label, peer_id])
    assert result == resource_id


def test_get_resource():
    label = get_label()
    resource_id = create_resource(label)
    peer_id = get_peer_id()
    result, error = run_aqua("getResource", [resource_id])
    assert result != None, error
    assert result["id"] == resource_id, error
    assert result["owner_peer_id"] == peer_id, error
    assert result["label"] == label, error


def test_register_record_unregister():
    relay = get_random_relay()
    label = get_label()
    value = "some_value"
    peer_id = get_peer_id()
    service_id = "id"

    resource_id = create_resource(label)
    result, error = run_aqua(
        "registerService", [resource_id, value, peer_id, service_id], relay)
    assert result, error

    # we want at least 1 successful response
    result, error = run_aqua("resolveResource", [resource_id, 1], relay)
    assert result != None, error

    assert len(result) == 1, "records not found"

    record = result[0]
    assert record["metadata"]["key_id"] == resource_id
    assert record["metadata"]["issued_by"] == peer_id
    assert record["metadata"]["peer_id"] == peer_id
    assert record["metadata"]["service_id"] == [service_id]

    result, error = run_aqua("unregisterService", [resource_id, peer_id],
                             relay)
    assert result, error

    result, error = run_aqua("resolveResource", [resource_id, 2], relay)
    assert result != None, error
    assert len(result) == 0


def test_register_unregister_remote_record():
    relay = get_random_relay()
    label = get_label()
    value = "some_value"
    issuer_peer_id = get_peer_id()
    peer_id = get_random_peer_id()
    service_id = "id"

    resource_id = create_resource(label)
    result, error = run_aqua(
        "registerService", [resource_id, value, peer_id, service_id], relay)
    assert result, error

    result, error = run_aqua("resolveResource", [resource_id, 2], relay)
    assert result != None, error

    assert len(result) == 1, "records not found"

    record = result[0]
    assert record["metadata"]["key_id"] == resource_id
    assert record["metadata"]["issued_by"] == issuer_peer_id
    assert record["metadata"]["peer_id"] == peer_id
    assert record["metadata"]["service_id"] == [service_id]

    result, error = run_aqua("unregisterService", [resource_id, peer_id],
                             relay)
    assert result, error

    result, error = run_aqua("resolveResource", [resource_id, 2], relay)
    assert result != None, error
    assert len(result) == 0
