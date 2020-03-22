import argparse
import logging
import pickle
import threading

# pylint: disable=import-error
import numpy as np
from xain_sdk import (
    ParticipantABC,
    TrainingInputABC,
    TrainingResultABC,
    configure_logging,
    run_participant,
)

LOG = logging.getLogger(__name__)


class TrainingInput(TrainingInputABC):
    def is_initialization_round(self) -> bool:
        return False


class TrainingResult(TrainingResultABC):
    def __init__(self, data: bytes):
        self.data = data

    def tobytes(self) -> bytes:
        return self.data


class Participant(ParticipantABC):
    def __init__(self, model: bytes) -> None:
        self.training_input = TrainingInput()
        self.training_result = TrainingResult(model)
        super(Participant, self).__init__()

    def deserialize_training_input(self, data: bytes) -> TrainingInput:
        return self.training_input

    def train_round(self, training_input: TrainingInput) -> TrainingResult:
        return self.training_result

    def init_weights(self) -> TrainingResult:
        return self.training_result


def participant_worker(participant, url, heartbeat_frequency, exit_event):
    try:
        run_participant(participant, url, heartbeat_frequency=heartbeat_frequency)
    except KeyboardInterrupt:
        exit_event.set()
    # pylint: disable=bare-except
    except:
        LOG.exception("participant exited with an error")
        exit_event.set()
    else:
        exit_event.set()


def main(
    size: int,
    number_of_participants: int,
    coordinator_url: str,
    heartbeat_frequency: int,
) -> None:
    """Entry point to start a participant."""
    weights = np.array([1] * size)
    training_result_data = int(0).to_bytes(4, byteorder="big") + pickle.dumps(weights)

    if number_of_participants < 2:
        participant = Participant(training_result_data)
        run_participant(
            participant, coordinator_url, heartbeat_frequency=heartbeat_frequency
        )
        return

    exit_event = threading.Event()
    threads = []
    for _ in range(0, number_of_participants):
        participant = Participant(training_result_data)
        thread = threading.Thread(
            target=participant_worker,
            args=(participant, coordinator_url, heartbeat_frequency, exit_event),
        )
        thread.daemon = True
        thread.start()
        threads.append(thread)

    def join_threads():
        for thread in threads:
            thread.join()
        LOG.info("all participants finished")
        exit_event.set()

    monitor = threading.Thread(target=join_threads)
    monitor.daemon = True
    monitor.start()
    exit_event.wait()

if __name__ == "__main__":
    # pylint: disable=invalid-name
    logging.basicConfig(
        format='%(asctime)s.%(msecs)03d %(levelname)-8s %(message)s',
        level=logging.DEBUG,
        datefmt='%Y-%m-%d %H:%M:%S')

    parser = argparse.ArgumentParser(description="Run dummy participants")
    parser.add_argument(
        "--number-of-participants",
        type=int,
        default=1,
        help="number of participants to start",
    )
    parser.add_argument(
        "--coordinator-url", type=str, required=True, help="URL of the coordinator",
    )
    parser.add_argument(
        "--model-size",
        type=int,
        # The default value corresponds roughly to a payload of 1MB
        default=125_000,
        help="Number of weights to use",
    )
    parser.add_argument(
        "--heartbeat-frequency",
        type=float,
        default=1,
        help="Frequency of the heartbeat in seconds",
    )
    parser.add_argument(
        "--verbose",
        action="store_true",
        help="Log the HTTP requests",
    )
    args = parser.parse_args()

    if args.verbose:
        configure_logging(level=logging.DEBUG,log_http_requests=True)
    else:
        configure_logging(level=logging.INFO ,log_http_requests=False)

    main(
        args.model_size,
        args.number_of_participants,
        args.coordinator_url,
        args.heartbeat_frequency,
    )
