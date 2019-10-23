import concurrent.futures
from pathlib import Path
from typing import Callable, Dict, List, Optional, Tuple

import tensorflow as tf
from absl import flags, logging

from xain.datasets import prep
from xain.fl.coordinator.controller import Controller
from xain.fl.logging.logging import create_summary_writer, write_summaries
from xain.fl.participant import ModelProvider, Participant
from xain.types import History, Metrics, Partition, Theta

from .aggregate import Aggregator, FederatedAveragingAgg

FLAGS = flags.FLAGS


class Coordinator:
    # pylint: disable-msg=too-many-arguments
    # pylint: disable=too-many-instance-attributes
    def __init__(
        self,
        controller,
        model_provider: ModelProvider,
        participants: List[Participant],
        C: float,
        E: int,
        xy_val: Partition,
        aggregator: Optional[Aggregator] = None,
    ) -> None:
        self.controller = controller
        self.model = model_provider.init_model()
        self.participants = participants
        self.C = C
        self.E = E
        self.xy_val = xy_val
        self.aggregator = aggregator if aggregator else FederatedAveragingAgg()
        self.epoch = 0  # Count training epochs

    # Common initialization happens implicitly: By updating the participant weights to
    # match the coordinator weights ahead of every training round we achieve common
    # initialization.
    def fit(
        self, num_rounds: int
    ) -> Tuple[History, List[List[History]], List[List[Dict]], List[List[Metrics]]]:
        # Initialize history; history coordinator
        hist_co: History = {"val_loss": [], "val_acc": []}
        # Train rounds; training history of selected participants
        hist_ps: List[List[History]] = []
        # History of optimizer configs in each round
        hist_opt_configs: List[List[Dict]] = []
        # History of participant metrics in each round
        hist_metrics: List[List[Metrics]] = []

        # Defining log directory and file writer for tensorboard logging
        val_log_dir: str = str(
            Path(FLAGS.output_dir).joinpath("tensorboard/coordinator")
        )
        summary_writer = create_summary_writer(logdir=val_log_dir)

        for r in range(num_rounds):
            # Determine who participates in this round
            num_indices = abs_C(self.C, self.num_participants())
            indices = self.controller.indices(num_indices)
            msg = f"Round {r+1}/{num_rounds}: Participants {indices}"
            logging.info(msg)

            # Train
            histories, opt_configs, train_metrics = self.fit_round(indices, self.E)
            hist_ps.append(histories)
            hist_opt_configs.append(opt_configs)
            hist_metrics.append(train_metrics)

            # Evaluate
            val_loss, val_acc = self.evaluate(self.xy_val)
            # Writing validation loss and accuracy into summary
            write_summaries(
                summary_writer=summary_writer,
                val_acc=val_acc,
                val_loss=val_loss,
                train_round=r,
            )
            hist_co["val_loss"].append(val_loss)
            hist_co["val_acc"].append(val_acc)

        logging.info(
            "TensorBoard coordinator validation logs saved: {}".format(val_log_dir)
        )
        logging.info(
            'Detailed analysis: call "tensorboard --logdir {}" from \
the console and open "localhost:6006" in a browser'.format(
                val_log_dir
            )
        )

        return hist_co, hist_ps, hist_opt_configs, hist_metrics

    def fit_round(
        self, indices: List[int], E: int
    ) -> Tuple[List[History], List[Dict], List[Metrics]]:
        theta = self.model.get_weights()
        participants = [self.participants[i] for i in indices]
        # Collect training results from the participants of this round
        theta_updates, histories, opt_configs, train_metrics = self.train_local_concurrently(
            theta, participants, E
        )
        # Aggregate training results
        theta_prime = self.aggregator.aggregate(theta_updates)
        # Update own model parameters
        self.model.set_weights(theta_prime)
        self.epoch += E
        return histories, opt_configs, train_metrics

    def train_local_sequentially(
        self, theta: Theta, participants: List[Participant], E: int
    ) -> Tuple[List[Tuple[Theta, int]], List[History], List[Dict], List[Metrics]]:
        """Train on each participant sequentially"""
        theta_updates: List[Tuple[Theta, int]] = []
        histories: List[History] = []
        opt_configs: List[Dict] = []
        train_metrics: List[Metrics] = []
        for participant in participants:
            # Train one round on this particular participant:
            # - Push current model parameters to this participant
            # - Train for a number of epochs
            # - Pull updated model parameters from participant
            theta_update, hist, opt_config = participant.train_round(
                theta, epochs=E, epoch_base=self.epoch
            )
            metrics = participant.metrics()
            theta_updates.append(theta_update)
            histories.append(hist)
            opt_configs.append(opt_config)
            train_metrics.append(metrics)
        return theta_updates, histories, opt_configs, train_metrics

    def train_local_concurrently(
        self, theta: Theta, participants: List[Participant], E: int
    ) -> Tuple[List[Tuple[Theta, int]], List[History], List[Dict], List[Metrics]]:
        """Train on each participant concurrently"""
        theta_updates: List[Tuple[Theta, int]] = []
        histories: List[History] = []
        opt_configs: List[Dict] = []
        train_metrics: List[Metrics] = []
        # Wait for all futures to complete
        with concurrent.futures.ThreadPoolExecutor() as executor:
            future_results = [
                executor.submit(train_local, p, theta, E, self.epoch)
                for p in participants
            ]
            concurrent.futures.wait(future_results)
            for future in future_results:
                theta_update, hist, opt_config, metrics = future.result()
                theta_updates.append(theta_update)
                histories.append(hist)
                opt_configs.append(opt_config)
                train_metrics.append(metrics)
        return theta_updates, histories, opt_configs, train_metrics

    def evaluate(self, xy_val: Partition) -> Tuple[float, float]:
        ds_val = prep.init_ds_val(xy_val)
        # Assume the validation `tf.data.Dataset` to yield exactly one batch containing
        # all examples in the validation set
        loss, accuracy = self.model.evaluate(ds_val, steps=1)
        return float(loss), float(accuracy)

    def num_participants(self) -> int:
        return len(self.participants)


def train_local(
    p: Participant, theta: Theta, epochs: int, epoch_base: int
) -> Tuple[Tuple[Theta, int], History, Dict, Metrics]:
    theta_update, history, opt_config = p.train_round(
        theta, epochs=epochs, epoch_base=epoch_base
    )
    metrics = p.metrics()
    return theta_update, history, opt_config, metrics


def abs_C(C: float, num_participants: int) -> int:
    return int(min(num_participants, max(1, C * num_participants)))


def create_evalueate_fn(
    orig_model: tf.keras.Model, xy_val: Partition
) -> Callable[[Theta], Tuple[float, float]]:
    ds_val = prep.init_ds_val(xy_val)
    model = tf.keras.models.clone_model(orig_model)
    # FIXME refactor model compilation
    model.compile(
        loss=tf.keras.losses.categorical_crossentropy,
        optimizer=tf.keras.optimizers.Adam(),
        metrics=["accuracy"],
    )

    def fn(theta: Theta) -> Tuple[float, float]:
        model.set_weights(theta)
        # Assume the validation `tf.data.Dataset` to yield exactly one batch containing
        # all examples in the validation set
        return model.evaluate(ds_val, steps=1)

    return fn


class SimpleCoordinator:
    def __init__(
        self,
        controller: Controller,
        num_participants: int,
        C: float,
        E: int,
        aggregator: Optional[Aggregator] = None,
    ):
        self.controller = controller
        self.num_participants = num_participants
        self.C = C
        self.E = E
        self.aggregator = aggregator if aggregator else FederatedAveragingAgg()
        self.epoch = 0  # Count training epochs
