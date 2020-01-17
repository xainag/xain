"""XAIN FL tests for gRPC coordinator"""

from concurrent import futures
import threading
from unittest import mock

import grpc
from numproto import ndarray_to_proto, proto_to_ndarray
import numpy as np
import pytest
from xain_proto.fl.coordinator_pb2 import (
    EndTrainingRoundRequest,
    EndTrainingRoundResponse,
    HeartbeatRequest,
    HeartbeatResponse,
    RendezvousReply,
    RendezvousRequest,
    StartTrainingRoundRequest,
    StartTrainingRoundResponse,
    State,
)
from xain_proto.fl.coordinator_pb2_grpc import add_CoordinatorServicer_to_server
from xain_proto.fl.hellonumproto_pb2 import NumProtoRequest
from xain_proto.fl.hellonumproto_pb2_grpc import NumProtoServerStub
from xain_sdk.participant_state_machine import (
    StateRecord,
    end_training_round,
    message_loop,
    rendezvous,
    start_participant,
    start_training_round,
)

from xain_fl.coordinator.coordinator import Coordinator
from xain_fl.coordinator.coordinator_grpc import CoordinatorGrpc
from xain_fl.coordinator.heartbeat import monitor_heartbeats
from xain_fl.coordinator.participants import Participants

from .store import TestStore


@pytest.mark.integration
def test_greeter_server(greeter_server):  # pylint: disable=unused-argument
    """[summary]

    .. todo:: Advance docstrings (https://xainag.atlassian.net/browse/XP-425)

    Args:
        greeter_server ([type]): [description]
    """

    with grpc.insecure_channel("localhost:50051") as channel:
        stub = NumProtoServerStub(channel)

        nda = np.arange(10)
        response = stub.SayHelloNumProto(NumProtoRequest(arr=ndarray_to_proto(nda)))

        response_nda = proto_to_ndarray(response.arr)

        assert np.array_equal(nda * 2, response_nda)


@pytest.mark.integration
def test_participant_rendezvous_accept(  # pylint: disable=unused-argument
    participant_stub, coordinator_service
):
    """[summary]

    .. todo:: Advance docstrings (https://xainag.atlassian.net/browse/XP-425)

    Args:
        participant_stub ([type]): [description]
        coordinator_service ([type]): [description]
    """

    response = participant_stub.Rendezvous(RendezvousRequest())

    assert response.reply == RendezvousReply.ACCEPT


@pytest.mark.integration
def test_participant_rendezvous_later(participant_stub):
    """[summary]

    .. todo:: Advance docstrings (https://xainag.atlassian.net/browse/XP-425)

    Args:
        participant_stub ([type]): [description]
    """

    # populate participants
    coordinator = Coordinator(minimum_participants_in_round=10, fraction_of_participants=1.0)
    required_participants = 10
    for i in range(required_participants):
        coordinator.participants.add(str(i))

    server = grpc.server(futures.ThreadPoolExecutor(max_workers=1))
    store = TestStore()
    add_CoordinatorServicer_to_server(CoordinatorGrpc(coordinator, store), server)
    server.add_insecure_port("localhost:50051")
    server.start()

    # try to rendezvous the 11th participant
    response = participant_stub.Rendezvous(RendezvousRequest())
    server.stop(0)

    assert response.reply == RendezvousReply.LATER


@pytest.mark.integration
def test_heartbeat(participant_stub, coordinator_service):
    """[summary]

    .. todo:: Advance docstrings (https://xainag.atlassian.net/browse/XP-425)

    Args:
        participant_stub ([type]): [description]
        coordinator_service ([type]): [description]
    """

    # first we need to rendezvous so that the participant is added to the list of participants
    _ = participant_stub.Rendezvous(RendezvousRequest())
    response = participant_stub.Heartbeat(HeartbeatRequest())

    # the Coordinator is initialised in conftest.py::coordinator_service with 10 participants
    # needed per round. so here we expect the HeartbeatResponse to have State.STANDBY
    # because we connected only one participant
    assert response == HeartbeatResponse()
    assert coordinator_service.coordinator.state == State.STANDBY


@pytest.mark.integration
def test_heartbeat_denied(participant_stub, coordinator_service):  # pylint: disable=unused-argument
    """[summary]

    .. todo:: Advance docstrings (https://xainag.atlassian.net/browse/XP-425)

    Args:
        participant_stub ([type]): [description]
        coordinator_service ([type]): [description]
    """

    # heartbeat requests are only allowed if the participant has already
    # rendezvous with the coordinator
    with pytest.raises(grpc.RpcError):
        response = participant_stub.Heartbeat(HeartbeatRequest())
        assert response.status_code == grpc.StatusCode.PERMISSION_DENIED


@mock.patch("xain_fl.coordinator.heartbeat.threading.Event.is_set", side_effect=[False, True])
@mock.patch("xain_fl.coordinator.heartbeat.time.sleep", return_value=None)
@mock.patch("xain_fl.coordinator.heartbeat.Coordinator.remove_participant")
def test_monitor_heartbeats(mock_participants_remove, _mock_sleep, _mock_event):
    """[summary]

    .. todo:: Advance docstrings (https://xainag.atlassian.net/browse/XP-425)

    Args:
        mock_participants_remove ([type]): [description]
        _mock_sleep ([type]): [description]
        _mock_event ([type]): [description]
    """

    participants = Participants()
    participants.add("participant_1")
    participants.participants["participant_1"].heartbeat_expires = 0

    coordinator = Coordinator()
    coordinator.participants = participants

    terminate_event = threading.Event()
    monitor_heartbeats(coordinator, terminate_event)

    mock_participants_remove.assert_called_once_with("participant_1")


@mock.patch("xain_fl.coordinator.heartbeat.threading.Event.is_set", side_effect=[False, True])
@mock.patch("xain_fl.coordinator.heartbeat.time.sleep", return_value=None)
def test_monitor_heartbeats_remove_participant(_mock_sleep, _mock_event):
    """[summary]

    .. todo:: Advance docstrings (https://xainag.atlassian.net/browse/XP-425)

    Args:
        _mock_sleep ([type]): [description]
        _mock_event ([type]): [description]
    """

    participants = Participants()
    participants.add("participant_1")
    participants.participants["participant_1"].heartbeat_expires = 0

    coordinator = Coordinator()
    coordinator.participants = participants

    terminate_event = threading.Event()
    monitor_heartbeats(coordinator, terminate_event)

    assert participants.len() == 0


@mock.patch(
    "xain_sdk.participant_state_machine.threading.Event.is_set", side_effect=[False, False, True]
)
@mock.patch("xain_sdk.participant_state_machine.time.sleep", return_value=None)
@mock.patch("xain_sdk.participant_state_machine.HeartbeatRequest")
def test_message_loop(mock_heartbeat_request, _mock_sleep, _mock_event):
    """[summary]

    .. todo:: Advance docstrings (https://xainag.atlassian.net/browse/XP-425)

    Args:
        mock_heartbeat_request ([type]): [description]
        _mock_sleep ([type]): [description]
        _mock_event ([type]): [description]
    """

    channel = mock.MagicMock()
    terminate_event = threading.Event()
    state_record = StateRecord()

    message_loop(channel, state_record, terminate_event)

    # check that the heartbeat is sent exactly twice
    mock_heartbeat_request.assert_has_calls([mock.call(), mock.call()])


@pytest.mark.integration
def test_start_training_round(coordinator_service):
    """[summary]

    .. todo:: Advance docstrings (https://xainag.atlassian.net/browse/XP-425)

    Args:
        coordinator_service ([type]): [description]
    """

    test_weights = [np.arange(10), np.arange(10, 20)]

    # set coordinator global model and hyper-params so that it needs only 1 participant
    coord = coordinator_service.coordinator
    coord.minimum_participants_in_round = 1
    coord.fraction_of_participants = 1.0
    coord.epochs = 5
    coord.epoch_base = 2
    coord.weights = test_weights
    coord.minimum_connected_participants = coord.get_minimum_connected_participants()

    # simulate a participant communicating with coordinator via channel
    with grpc.insecure_channel("localhost:50051") as channel:
        # we need to rendezvous before we can send any other requests
        rendezvous(channel)
        # call StartTrainingRound service method on coordinator
        weights, epochs, epoch_base = start_training_round(channel)

    # check global model received
    assert epochs == 5
    assert epoch_base == 2
    np.testing.assert_equal(weights, test_weights)


@pytest.mark.integration
def test_start_training_round_denied(  # pylint: disable=unused-argument
    participant_stub, coordinator_service
):
    """[summary]

    .. todo:: Advance docstrings (https://xainag.atlassian.net/browse/XP-425)

    Args:
        participant_stub ([type]): [description]
        coordinator_service ([type]): [description]
    """

    # start training requests are only allowed if the participant has already
    # rendezvous with the coordinator
    with pytest.raises(grpc.RpcError):
        response = participant_stub.StartTrainingRound(StartTrainingRoundRequest())
        assert response.status_code == grpc.StatusCode.PERMISSION_DENIED


@pytest.mark.integration
def test_start_training_round_failed_precondition(  # pylint: disable=unused-argument
    participant_stub, coordinator_service
):
    """[summary]

    .. todo:: Advance docstrings (https://xainag.atlassian.net/browse/XP-425)

    Args:
        participant_stub ([type]): [description]
        coordinator_service ([type]): [description]
    """

    # start training requests are only allowed if the coordinator is in ROUND state.
    # Since we need 10 participants to be connected (see conftest.py::coordinator_service)
    # the StartTrainingRoundRequest is expected to fail
    participant_stub.Rendezvous(RendezvousRequest())
    with pytest.raises(grpc.RpcError):
        response = participant_stub.StartTrainingRound(StartTrainingRoundRequest())
        assert response.status_code == grpc.StatusCode.FAILED_PRECONDITION


@pytest.mark.integration
def test_end_training_round(coordinator_service):
    """[summary]

    .. todo:: Advance docstrings (https://xainag.atlassian.net/browse/XP-425)

    Args:
        coordinator_service ([type]): [description]
    """

    assert coordinator_service.coordinator.round.updates == {}

    # simulate trained local model data
    test_weights, number_samples = [np.arange(20, 30), np.arange(30, 40)], 2
    test_metrics = {"metric": np.arange(10, 20)}

    with grpc.insecure_channel("localhost:50051") as channel:
        # we first need to rendezvous before we can send any other request
        rendezvous(channel)
        # call EndTrainingRound service method on coordinator
        end_training_round(channel, test_weights, number_samples, test_metrics)
    # check local model received...

    assert len(coordinator_service.coordinator.round.updates) == 1

    round_ = coordinator_service.coordinator.round

    # first the weights update
    _, round_update = round_.updates.popitem()
    update_item1, update_item2 = round_update["weight_update"]
    assert update_item2 == number_samples
    np.testing.assert_equal(update_item1, test_weights)

    round_update_metrics = round_update["metrics"]
    assert round_update_metrics.keys() == test_metrics.keys()
    for key, values in test_metrics.items():
        np.testing.assert_equal(round_update_metrics[key], values)


@pytest.mark.integration
def test_end_training_round_duplicated_updates(  # pylint: disable=unused-argument
    coordinator_service, participant_stub
):
    """[summary]

    .. todo:: Advance docstrings (https://xainag.atlassian.net/browse/XP-425)

    Args:
        coordinator_service ([type]): [description]
        participant_stub ([type]): [description]
    """

    # participant can only send updates once in a single round
    participant_stub.Rendezvous(RendezvousRequest())

    participant_stub.EndTrainingRound(EndTrainingRoundRequest())

    with pytest.raises(grpc.RpcError):
        response = participant_stub.EndTrainingRound(EndTrainingRoundRequest())
        assert response.status_code == grpc.StatusCode.ALREADY_EXISTS


@pytest.mark.integration
def test_end_training_round_denied(  # pylint: disable=unused-argument
    participant_stub, coordinator_service
):
    """[summary]

    .. todo:: Advance docstrings (https://xainag.atlassian.net/browse/XP-425)

    Args:
        participant_stub ([type]): [description]
        coordinator_service ([type]): [description]
    """

    # heartbeat requests are only allowed if the participant has already
    # rendezvous with the coordinator
    with pytest.raises(grpc.RpcError):
        response = participant_stub.EndTrainingRound(EndTrainingRoundRequest())
        assert response.status_code == grpc.StatusCode.PERMISSION_DENIED


@pytest.mark.integration
def test_full_training_round(
    participant_stubs, coordinator_service
):  # pylint: disable=unused-argument
    """Run a complete training round with multiple participants.
    """
    # Initialize the coordinator with dummy weights, otherwise, the
    # aggregated weights at the end of the round are an empty array.
    dummy_weights = np.ndarray([1, 2, 3, 4])
    coordinator_service.coordinator.weights = dummy_weights
    weights_proto = [ndarray_to_proto(nda) for nda in dummy_weights]

    # Create 10 partipants
    participants = [next(participant_stubs) for _ in range(0, 10)]

    # 9 participants out of 10 connect to the coordinator, so it stays
    # in STANDBY and accepts the connections.
    for participant in participants[:-1]:
        response = participant.Rendezvous(RendezvousRequest())
        assert response.reply == RendezvousReply.ACCEPT

        response = participant.Heartbeat(HeartbeatRequest())
        assert response == HeartbeatResponse(state=State.STANDBY, round=0)

    assert coordinator_service.coordinator.state == State.STANDBY
    assert coordinator_service.coordinator.current_round == 0

    # The 10th participant connect, so the coordinator switches to ROUND>
    last_participant = participants[-1]
    response = last_participant.Rendezvous(RendezvousRequest())
    assert response.reply == RendezvousReply.ACCEPT

    assert coordinator_service.coordinator.state == State.ROUND
    assert coordinator_service.coordinator.current_round == 1

    response = last_participant.Heartbeat(HeartbeatRequest())
    assert response == HeartbeatResponse(state=State.ROUND, round=1)

    # The initial 9 participants send another heatbeat request.
    for participant in participants[:-1]:
        response = participant.Heartbeat(HeartbeatRequest(state=State.STANDBY, round=0))
    assert response == HeartbeatResponse(state=State.ROUND, round=1)

    # The participants start training
    for participant in participants:
        response = participant.StartTrainingRound(StartTrainingRoundRequest())
        assert response == StartTrainingRoundResponse(
            weights=weights_proto,
            epochs=coordinator_service.coordinator.epochs,
            epoch_base=coordinator_service.coordinator.epoch_base,
        )

    # The first 9th participants end training
    for participant in participants[:-1]:
        response = participant.EndTrainingRound(
            EndTrainingRoundRequest(weights=weights_proto, number_samples=1)
        )
        assert response == EndTrainingRoundResponse()

    assert not coordinator_service.coordinator.round.is_finished()
    coordinator_service.store.assert_didnt_write(1)

    # The last participant finishes training
    response = last_participant.EndTrainingRound(EndTrainingRoundRequest())
    assert response == EndTrainingRoundResponse()
    # Make sure we wrote the results for the given round
    coordinator_service.store.assert_wrote(1, coordinator_service.coordinator.weights)


@pytest.mark.integration
@pytest.mark.slow
def test_start_participant(mock_coordinator_service):
    """[summary]

    .. todo:: Advance docstrings (https://xainag.atlassian.net/browse/XP-425)

    Args:
        mock_coordinator_service ([type]): [description]
    """

    init_weight = [np.arange(10), np.arange(10, 20)]
    mock_coordinator_service.coordinator.weights = init_weight

    # mock a local participant with a constant train_round function
    with mock.patch("xain_sdk.participant_state_machine.Participant") as mock_obj:
        mock_local_part = mock_obj.return_value
        mock_local_part.train_round.return_value = init_weight, 1, {}

        start_participant(mock_local_part, "localhost:50051")

        coord = mock_coordinator_service.coordinator
        assert coord.state == State.FINISHED

        # coordinator set to 2 round for good measure, but the resulting
        # aggregated weights are the same as a single round
        assert coord.current_round == 2

        # expect weight aggregated by summation - see mock_coordinator_service
        np.testing.assert_equal(coord.weights, [np.arange(start=10, stop=29, step=2)])
