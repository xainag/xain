import threading
import time
from typing import Tuple

import grpc
from numproto import ndarray_to_proto, proto_to_ndarray

from xain.grpc import coordinator_pb2, coordinator_pb2_grpc
from xain.types import History, Metrics, Theta

RETRY_TIMEOUT = 5
HEARTBEAT_TIME = 10


def heartbeat(channel, terminate_event):
    stub = coordinator_pb2_grpc.CoordinatorStub(channel)
    while not terminate_event.is_set():
        reply = stub.Heartbeat(coordinator_pb2.HeartbeatRequest())
        print(f"Participant received: {type(reply)}")
        time.sleep(HEARTBEAT_TIME)


def rendezvous(channel):
    stub = coordinator_pb2_grpc.CoordinatorStub(channel)

    response = coordinator_pb2.RendezvousResponse.LATER

    while response == coordinator_pb2.RendezvousResponse.LATER:
        reply = stub.Rendezvous(coordinator_pb2.RendezvousRequest())
        if reply.response == coordinator_pb2.RendezvousResponse.ACCEPT:
            print("Participant received: ACCEPT")
        elif reply.response == coordinator_pb2.RendezvousResponse.LATER:
            print(f"Participant received: LATER. Retrying in {RETRY_TIMEOUT}s")
            time.sleep(RETRY_TIMEOUT)

        response = reply.response


def start_training(channel) -> Tuple[Theta, int, int]:
    stub = coordinator_pb2_grpc.CoordinatorStub(channel)
    req = coordinator_pb2.StartTrainingRequest()
    # send request to start training
    reply = stub.StartTraining(req)
    print(f"Participant received: {type(reply)}")
    theta, epochs, epoch_base = reply.theta, reply.epochs, reply.epoch_base
    return [proto_to_ndarray(pnda) for pnda in theta], epochs, epoch_base


def end_training(
    channel, theta_update: Tuple[Theta, int], history: History, metrics: Metrics
):
    stub = coordinator_pb2_grpc.CoordinatorStub(channel)
    # build request starting with theta update

    # nda1, nda2 = np.arange(20, 30), np.arange(30, 40)
    # num = 2
    # print(f"Participant requests: theta' {nda1}, {nda2}; {num} examples")
    # theta_prime = [ndarray_to_proto(nda1), ndarray_to_proto(nda2)]
    # theta_update = coordinator_pb2.EndTrainingRequest.ThetaUpdate(
    #     theta_prime=theta_prime, num_examples=num)
    theta, num = theta_update
    theta_update = coordinator_pb2.EndTrainingRequest.ThetaUpdate(
        theta_prime=[ndarray_to_proto(nda) for nda in theta], num_examples=num
    )
    # history data

    # k1, k2 = "aaa", "bbb"
    # v1, v2 = [1.1, 2.1], [3.1, 4.1]
    # print(f"History data: {k1}: {v1}; {k2}: {v2}")
    # his = {
    #     k1: coordinator_pb2.EndTrainingRequest.HistoryValue(values=v1),
    #     k2: coordinator_pb2.EndTrainingRequest.HistoryValue(values=v2),
    # }
    h = {
        k: coordinator_pb2.EndTrainingRequest.HistoryValue(values=v)
        for k, v in history.items()
    }
    # metrics

    # cid, vbc = 1, [3, 4, 5]
    # print(f"Metrics: cid {cid}, vol by class {vbc}")
    # met = coordinator_pb2.EndTrainingRequest.Metrics(cid=cid, vol_by_class=vbc)
    cid, vbc = metrics
    m = coordinator_pb2.EndTrainingRequest.Metrics(cid=cid, vol_by_class=vbc)

    # assemble req

    # req = coordinator_pb2.EndTrainingRequest(
    #     theta_update=theta_update, history=his, metrics=met
    # )
    req = coordinator_pb2.EndTrainingRequest(
        theta_update=theta_update, history=h, metrics=m
    )
    # send request to end training
    reply = stub.EndTraining(req)
    print(f"Participant received: {type(reply)}")


def run():
    # create a channel to the coordinator
    channel = grpc.insecure_channel("localhost:50051")

    # rendezvous with the coordinator
    rendezvous(channel)

    # start the heartbeat in a different thread
    terminate_event = threading.Event()
    heartbeat_thread = threading.Thread(
        target=heartbeat, args=(channel, terminate_event)
    )
    heartbeat_thread.start()

    # sample training round
    theta, _, _ = start_training(channel)
    # end training with some default values; in actual training you'd pass
    # along the theta_update, history and metrics to end the round
    end_training(channel, (theta, 0), {}, (0, []))

    try:
        # never returns unless there is an exception
        heartbeat_thread.join()
    except KeyboardInterrupt:
        terminate_event.set()
        channel.close()


if __name__ == "__main__":
    run()
