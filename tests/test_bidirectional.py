import time
import traceback
import pytest
from iroh_py import Node
from multiprocessing import Process, Queue
from functools import partial

def multi_process_test(*funcs):
    q = Queue()
    processes = []
    for func in funcs:
        p = Process(target=func, args=(q,))
        p.start()
        processes.append(p)

    for p in processes:
        p.join()

    # Check for exceptions in subprocesses
    failures = []
    while not q.empty():
        result = q.get()
        if result is not None:
            exception, traceback = result
            failures.append(f"Subprocess failed with: {exception}\n{traceback}")
    
    # If any subprocess failed, fail the test
    if failures:
        pytest.fail("\n".join(failures))

def connect(q: Queue, seed: int, peer_id: str):
    try:
        node = Node.with_seed(num_micro_batches=2, seed=seed)
        print(f"Node {node.node_id()}")
        assert node.node_id() is not None
        assert not (node.can_recv() or node.can_send() or node.is_ready())
        node.connect(peer_id)
        while not node.is_ready():
            time.sleep(1)
        assert node.can_recv() and node.can_send() and node.is_ready()
        # Signal success
        q.put(None)
    except Exception as e:
        # Send back the exception and traceback
        q.put((e, traceback.format_exc()))

def send_recv(q: Queue, seed: int, peer_id: str):
    try:
        node = Node.with_seed(num_micro_batches=1, seed=seed)
        print(f"Node {node.node_id()}")
        assert node.node_id() is not None
        assert not (node.can_recv() or node.can_send() or node.is_ready())
        node.connect(peer_id)
        while not node.is_ready():
            time.sleep(1)
        assert node.can_recv() and node.can_send() and node.is_ready()
        sent = b"Hello, world!"
        node.isend(sent, 0).wait()
        rcvd = node.irecv(0).wait()
        assert rcvd == sent
        # Signal success
        q.put(None)
    except Exception as e:
        # Send back the exception and traceback
        q.put((e, traceback.format_exc()))

args = [
    (42, "da1d3d33264bebd2ff215473064fb11f4c7ceb4b820a3868c2c792d27e205691"),
    (69, "9bdb607f02802cdd126290cfa1e025e4c13bbdbb347a70edeace584159303454"),
]

def test_bidirectional_connect():
    multi_process_test(*[partial(connect, seed=args[i][0], peer_id=args[i][1]) for i in range(len(args))])

def test_bidirectional_send_recv():
    multi_process_test(*[partial(send_recv, seed=args[i][0], peer_id=args[i][1]) for i in range(len(args))])
