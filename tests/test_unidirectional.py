import time
import traceback
import pytest
from iroh_py import Node
from multiprocessing import Process, Queue

def run_receiver(q: Queue):
    try:
        node = Node.with_seed(num_micro_batches=2, seed=42)
        assert node.node_id() is not None
        assert not node.can_recv() and not node.can_send() and not node.is_ready()
        while not node.can_recv():
            time.sleep(1)
        assert node.can_recv()
        q.put(None)
        node.close()
    except Exception as e:
        q.put((e, traceback.format_exc()))

def run_sender(q: Queue):
    try:
        node = Node(num_micro_batches=2)
        assert node.node_id() is not None
        assert not node.can_recv() and not node.can_send() and not node.is_ready()
        peer_id = "9bdb607f02802cdd126290cfa1e025e4c13bbdbb347a70edeace584159303454"
        node.connect(peer_id)
        while not node.can_send():
            time.sleep(1)
        assert node.can_send()
        q.put(None)
        node.close()
    except Exception as e:
        q.put((e, traceback.format_exc()))

def test_unidirectional():
    q = Queue()
    
    receiver = Process(target=run_receiver, args=(q,))
    sender = Process(target=run_sender, args=(q,))

    receiver.start()
    sender.start()

    receiver.join()
    sender.join()
    
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
