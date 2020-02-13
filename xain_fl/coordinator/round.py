"""XAIN FL Rounds"""

from typing import Dict, List, Tuple

from numpy import ndarray

from xain_fl.tools.exceptions import DuplicatedUpdateError


class Round:
    """Class to manage the state of a single round.
    This class contains the logic to handle all updates sent by the
    participants during a round and does some sanity checks like preventing the
    same participant to submit multiple updates during a single round.

    Args:
        participant_ids: The list of IDs of the participants selected
            to participate in this round.
    """

    def __init__(self, participant_ids: List[str]) -> None:
        self.participant_ids = participant_ids
        self.updates: Dict[str, Dict] = {}

    def add_selected(self, more_ids: List[str]) -> None:
        """Add to the collection of selected participants.

        Args:
            more_ids: ids of participants to add.
        """

        self.participant_ids.extend(more_ids)

    def remove_selected(self, participant_id: str) -> None:
        """Remove from the collection of selected participants.

        Args:
            participant_id: id of participant to remove.
        """

        try:
            self.participant_ids.remove(participant_id)
        except ValueError:
            pass

    def add_updates(
        self, participant_id: str, model_weights: ndarray, aggregation_data: int,
    ) -> None:
        """Add a participant's update for the round.

        Args:
            participant_id: The id of the participant making the request.

            model_weights: The updated model weights.

            aggregation_data: Meta data for aggregation.

        Raises:
            DuplicatedUpdateError: If the participant already submitted his update this round.
        """

        if participant_id in self.updates.keys():
            raise DuplicatedUpdateError(
                f"Participant {participant_id} already submitted the update for this round."
            )

        self.updates[participant_id] = {
            "model_weights": model_weights,
            "aggregation_data": aggregation_data,
        }

    def is_finished(self) -> bool:
        """Check if all the required participants submitted their updates this round.
        If all participants submitted their updates the round is considered finished.

        Returns:
            `True` if all participants submitted their updates this
            round. `False` otherwise.
        """

        return all(id in self.updates for id in self.participant_ids)

    def get_weight_updates(self) -> Tuple[List[ndarray], List[int]]:
        """Get a list of all participants weight updates.
        This list will usually be used by the aggregation function.

        Returns:
            The lists of model weights and aggregation meta data from all participants.
        """

        updates = [self.updates[id] for id in self.participant_ids]
        return (
            [upd["model_weights"] for upd in updates],
            [upd["aggregation_data"] for upd in updates],
        )
