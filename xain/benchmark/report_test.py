import os

import pytest
from absl import flags

from xain.helpers import sha1, storage

from . import report

FLAGS = flags.FLAGS


def test_read_accuracies_from_results_file(monkeypatch):
    # Prepare
    json_data = {
        "name": "foo",
        "unitary_learning": {"acc": 0.1},
        "federated_learning": {"acc": 0.2},
    }

    def mock_read_json(_: str):
        return json_data

    monkeypatch.setattr(storage, "read_json", mock_read_json)

    expected_data = (
        json_data["name"],
        json_data["unitary_learning"]["acc"],
        json_data["federated_learning"]["acc"],
    )

    # Execute
    actual_data = report.read_accuracies_from_results_file("any.json")

    # Assert
    assert expected_data == actual_data


@pytest.mark.integration
def test_read_accuracies_from_group(monkeypatch, group_name, results_dir):
    # Prepare
    other_group_name = "other_group"
    assert group_name != other_group_name  # just in case

    group_dir = os.path.join(results_dir, group_name)
    other_group_dir = os.path.join(results_dir, other_group_name)

    files = [
        f"{group_dir}/task_1/results.json",
        f"{group_dir}/task_2/results.json",
        f"{other_group_dir}/task_1/results.json",
        f"{other_group_dir}/task_2/results.json",
    ]

    for fname in files:
        dname = os.path.dirname(fname)
        os.makedirs(dname)
        with open(fname, "x") as f:
            f.write("content not relevant")
            f.close()

    expected_results = files[:2]

    def mock_read_accuracies_from_results_file(fname):
        return fname

    monkeypatch.setattr(
        report,
        "read_accuracies_from_results_file",
        mock_read_accuracies_from_results_file,
    )

    # Execute
    actual_results = report.read_accuracies_from_group(group_dir)

    # Assert
    assert actual_results == expected_results


@pytest.mark.integration
def test_plot_iid_noniid_comparison(output_dir, group_name, monkeypatch):
    # Prepare
    data = [
        (
            "unitary",
            [0.96, 0.90, 0.81, 0.72, 0.63, 0.54, 0.45, 0.36, 0.27, 0.18, 0.09],
            range(1, 12, 1),
        ),
        (
            "federated",
            [0.92, 0.89, 0.87, 0.85, 0.83, 0.81, 0.80, 0.79, 0.78, 0.77, 0.77],
            range(1, 12, 1),
        ),
    ]
    fname = f"plot_{group_name}.png"
    expected_filepath = os.path.join(output_dir, fname)
    expected_sha1 = "4b9fb44d7d3f92889ada5d59bb74d21a34a5fdaa"

    xticks_locations = range(1, 12, 1)
    xticks_labels = [chr(i) for i in range(65, 77, 1)]  # A, B, ..., K

    def mock_prepare_iid_noniid_comparison_data(_: str):
        return (data, (xticks_locations, xticks_labels))

    monkeypatch.setattr(
        report,
        "prepare_iid_noniid_comparison_data",
        mock_prepare_iid_noniid_comparison_data,
    )

    # Execute
    actual_filepath = report.plot_iid_noniid_comparison()

    # If any error occurs we will be able to look at the plot. If the the ploting
    # logic is changed the file under this path can be used to get the new hash
    # after evaluating the rendered plot
    print(actual_filepath)

    # Assert
    assert expected_filepath == actual_filepath
    assert expected_sha1 == sha1.checksum(actual_filepath), "Checksum not matching"


@pytest.mark.integration
def test_plot_accuracies(output_dir):
    # Prepare
    data = [
        (
            "unitary",
            [0.96, 0.90, 0.81, 0.72, 0.63, 0.54, 0.45, 0.36, 0.27, 0.18, 0.09],
            range(1, 12, 1),
        ),
        (
            "federated",
            [0.92, 0.89, 0.87, 0.85, 0.83, 0.81, 0.80, 0.79, 0.78, 0.77, 0.77],
            range(1, 12, 1),
        ),
    ]
    fname = "myplot.png"
    expected_filepath = os.path.join(output_dir, fname)
    expected_sha1 = "457baa8179f08f06c4e60213eb0bbbe79a4f9d3e"

    # Execute
    actual_filepath = report.plot_accuracies(data=data, fname=fname)

    # If any error occurs we will be able to look at the plot. If the the ploting
    # logic is changed the file under this path can be used to get the new hash
    # after evaluating the rendered plot
    print(actual_filepath)

    # Assert
    assert expected_filepath == actual_filepath
    assert expected_sha1 == sha1.checksum(actual_filepath), "Checksum not matching"
