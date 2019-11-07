import os
from tempfile import TemporaryDirectory
from time import strftime
from typing import Dict, List, Optional

from absl import flags, logging

from benchmarks.ops import docker, results, run
from xain.helpers import storage

from .task import Task, UnitaryVisionTask, VisionTask

FLAGS = flags.FLAGS


class Benchmark:
    def __init__(self, tasks: List[Task], aggregation_name: str, runner: str = "ec2"):
        self.tasks = tasks
        self.aggregation_name = aggregation_name
        self.runner = runner


benchmarks: Dict[str, Benchmark] = {
    #
    # ##############################
    # UL/FL CPP
    # ##############################
    #
    "flul-fashion-mnist-100p-iid-balanced": Benchmark(
        tasks=[
            UnitaryVisionTask(
                name="unitary",
                label="Unitary",
                dataset_name="fashion-mnist-100p-iid-balanced",
                model_name="orig_cnn",
                E=100,
                partition_id=0,
                instance_cores=8,
            ),
            VisionTask(
                name="federated",
                label="Federated",
                dataset_name="fashion-mnist-100p-iid-balanced",
                model_name="orig_cnn",
                R=100,
                E=1,
                instance_cores=32,
                timeout=120,
            ),
        ],
        aggregation_name="flul-aggregation",
    ),
    "flul-fashion-mnist-100p-noniid-05cpp": Benchmark(
        tasks=[
            UnitaryVisionTask(
                name="unitary",
                label="Unitary",
                dataset_name="fashion-mnist-100p-noniid-05cpp",
                model_name="orig_cnn",
                E=100,
                partition_id=0,
                instance_cores=8,
            ),
            VisionTask(
                name="federated",
                label="Federated",
                dataset_name="fashion-mnist-100p-noniid-05cpp",
                model_name="orig_cnn",
                R=100,
                E=1,
                instance_cores=32,
                timeout=120,
            ),
        ],
        aggregation_name="flul-aggregation",
    ),
    #
    # ##############################
    # UL/FL VOL
    # ##############################
    #
    "flul-fashion-mnist-100p-b1_045": Benchmark(
        tasks=[
            UnitaryVisionTask(
                name="unitary_p0",
                label='"Unitary (n=30)"',  # low volume
                dataset_name="fashion-mnist-100p-b1_045",
                model_name="orig_cnn",
                E=100,
                B=16,
                partition_id=0,
                instance_cores=16,
                timeout=120,
            ),
            UnitaryVisionTask(
                name="unitary_p99",
                label='"Unitary (n=2356)"',  # high volume
                dataset_name="fashion-mnist-100p-b1_045",
                model_name="orig_cnn",
                E=100,
                B=16,
                partition_id=99,
                instance_cores=16,
                timeout=120,
            ),
            VisionTask(
                name="federated",
                label="Federated",
                dataset_name="fashion-mnist-100p-b1_045",
                model_name="orig_cnn",
                R=100,
                E=1,
                B=16,
                instance_cores=32,
                timeout=120,
            ),
        ],
        aggregation_name="flul-aggregation",
    ),
    #
    # ##############################
    # CPP CONTINUUM
    # ##############################
    #
    "cpp-fashion-mnist-100p": Benchmark(
        tasks=[
            VisionTask(
                name="federated-01cpp",
                label="01cpp",
                dataset_name="fashion-mnist-100p-noniid-01cpp",
                model_name="orig_cnn",
                R=100,
                E=1,
                instance_cores=32,
                timeout=180,
            ),
            VisionTask(
                name="federated-02cpp",
                label="02cpp",
                dataset_name="fashion-mnist-100p-noniid-02cpp",
                model_name="orig_cnn",
                R=100,
                E=1,
                instance_cores=32,
                timeout=180,
            ),
            VisionTask(
                name="federated-03cpp",
                label="03cpp",
                dataset_name="fashion-mnist-100p-noniid-03cpp",
                model_name="orig_cnn",
                R=100,
                E=1,
                instance_cores=32,
                timeout=180,
            ),
            VisionTask(
                name="federated-04cpp",
                label="04cpp",
                dataset_name="fashion-mnist-100p-noniid-04cpp",
                model_name="orig_cnn",
                R=100,
                E=1,
                instance_cores=32,
                timeout=180,
            ),
            VisionTask(
                name="federated-05cpp",
                label="05cpp",
                dataset_name="fashion-mnist-100p-noniid-05cpp",
                model_name="orig_cnn",
                R=100,
                E=1,
                instance_cores=32,
                timeout=180,
            ),
            VisionTask(
                name="federated-06cpp",
                label="06cpp",
                dataset_name="fashion-mnist-100p-noniid-06cpp",
                model_name="orig_cnn",
                R=100,
                E=1,
                instance_cores=32,
                timeout=180,
            ),
            VisionTask(
                name="federated-07cpp",
                label="07cpp",
                dataset_name="fashion-mnist-100p-noniid-07cpp",
                model_name="orig_cnn",
                R=100,
                E=1,
                instance_cores=32,
                timeout=180,
            ),
            VisionTask(
                name="federated-08cpp",
                label="08cpp",
                dataset_name="fashion-mnist-100p-noniid-08cpp",
                model_name="orig_cnn",
                R=100,
                E=1,
                instance_cores=32,
                timeout=180,
            ),
            VisionTask(
                name="federated-09cpp",
                label="09cpp",
                dataset_name="fashion-mnist-100p-noniid-09cpp",
                model_name="orig_cnn",
                R=100,
                E=1,
                instance_cores=32,
                timeout=180,
            ),
            VisionTask(
                name="federated-balanced",
                label="balanced",
                dataset_name="fashion-mnist-100p-iid-balanced",
                model_name="orig_cnn",
                R=100,
                E=1,
                instance_cores=32,
                timeout=180,
            ),
            UnitaryVisionTask(
                name="unitary-01cpp",
                label="01cpp",
                dataset_name="fashion-mnist-100p-noniid-01cpp",
                model_name="orig_cnn",
                E=100,
                instance_cores=16,
            ),
            UnitaryVisionTask(
                name="unitary-02cpp",
                label="02cpp",
                dataset_name="fashion-mnist-100p-noniid-02cpp",
                model_name="orig_cnn",
                E=100,
                instance_cores=16,
            ),
            UnitaryVisionTask(
                name="unitary-03cpp",
                label="03cpp",
                dataset_name="fashion-mnist-100p-noniid-03cpp",
                model_name="orig_cnn",
                E=100,
                instance_cores=16,
            ),
            UnitaryVisionTask(
                name="unitary-04cpp",
                label="04cpp",
                dataset_name="fashion-mnist-100p-noniid-04cpp",
                model_name="orig_cnn",
                E=100,
                instance_cores=16,
            ),
            UnitaryVisionTask(
                name="unitary-05cpp",
                label="05cpp",
                dataset_name="fashion-mnist-100p-noniid-05cpp",
                model_name="orig_cnn",
                E=100,
                instance_cores=16,
            ),
            UnitaryVisionTask(
                name="unitary-06cpp",
                label="06cpp",
                dataset_name="fashion-mnist-100p-noniid-06cpp",
                model_name="orig_cnn",
                E=100,
                instance_cores=16,
            ),
            UnitaryVisionTask(
                name="unitary-07cpp",
                label="07cpp",
                dataset_name="fashion-mnist-100p-noniid-07cpp",
                model_name="orig_cnn",
                E=100,
                instance_cores=16,
            ),
            UnitaryVisionTask(
                name="unitary-08cpp",
                label="08cpp",
                dataset_name="fashion-mnist-100p-noniid-08cpp",
                model_name="orig_cnn",
                E=100,
                instance_cores=16,
            ),
            UnitaryVisionTask(
                name="unitary-09cpp",
                label="09cpp",
                dataset_name="fashion-mnist-100p-noniid-09cpp",
                model_name="orig_cnn",
                E=100,
                instance_cores=16,
            ),
            UnitaryVisionTask(
                name="unitary-balanced",
                label="balanced",
                dataset_name="fashion-mnist-100p-iid-balanced",
                model_name="orig_cnn",
                E=100,
                instance_cores=16,
            ),
        ],
        aggregation_name="cpp-aggregation",
    ),
    #
    # ##############################
    # VOL CONTINUUM
    # ##############################
    #
    ## TODO
    #
    # ##############################
    # C LOW-TO-HIGH
    # ##############################
    #
    "C-fashion-mnist-100p-noniid-03cpp": Benchmark(
        tasks=[
            VisionTask(
                name="federated-C0_01",
                label="C=0.01",
                dataset_name="fashion-mnist-100p-noniid-03cpp",
                model_name="orig_cnn",
                R=50,
                E=1,
                C=0.01,
                instance_cores=32,
                timeout=120,
            ),
            VisionTask(
                name="federated-C0_03",
                label="C=0.03",
                dataset_name="fashion-mnist-100p-noniid-03cpp",
                model_name="orig_cnn",
                R=50,
                E=1,
                C=0.03,
                instance_cores=32,
                timeout=120,
            ),
            VisionTask(
                name="federated-C0_10",
                label="C=0.1",
                dataset_name="fashion-mnist-100p-noniid-03cpp",
                model_name="orig_cnn",
                R=50,
                E=1,
                C=0.1,
                instance_cores=32,
                timeout=120,
            ),
            VisionTask(
                name="federated-C0_20",
                label="C=0.2",
                dataset_name="fashion-mnist-100p-noniid-03cpp",
                model_name="orig_cnn",
                R=50,
                E=1,
                C=0.2,
                instance_cores=32,
                timeout=240,
            ),
        ],
        aggregation_name="flul-aggregation",
    ),
    "C-fashion-mnist-100p-noniid-05cpp": Benchmark(
        tasks=[
            VisionTask(
                name="federated-C0_01",
                label="C=0.01",
                dataset_name="fashion-mnist-100p-noniid-05cpp",
                model_name="orig_cnn",
                R=50,
                E=1,
                C=0.01,
                instance_cores=32,
                timeout=120,
            ),
            VisionTask(
                name="federated-C0_03",
                label="C=0.03",
                dataset_name="fashion-mnist-100p-noniid-05cpp",
                model_name="orig_cnn",
                R=50,
                E=1,
                C=0.03,
                instance_cores=32,
                timeout=120,
            ),
            VisionTask(
                name="federated-C0_10",
                label="C=0.1",
                dataset_name="fashion-mnist-100p-noniid-05cpp",
                model_name="orig_cnn",
                R=50,
                E=1,
                C=0.1,
                instance_cores=32,
                timeout=120,
            ),
            VisionTask(
                name="federated-C0_20",
                label="C=0.2",
                dataset_name="fashion-mnist-100p-noniid-05cpp",
                model_name="orig_cnn",
                R=50,
                E=1,
                C=0.2,
                instance_cores=32,
                timeout=240,
            ),
        ],
        aggregation_name="flul-aggregation",
    ),
    "C-fashion-mnist-100p-noniid-09cpp": Benchmark(
        tasks=[
            VisionTask(
                name="federated-C0_01",
                label="C=0.01",
                dataset_name="fashion-mnist-100p-noniid-09cpp",
                model_name="orig_cnn",
                R=50,
                E=1,
                C=0.01,
                instance_cores=32,
                timeout=120,
            ),
            VisionTask(
                name="federated-C0_03",
                label="C=0.03",
                dataset_name="fashion-mnist-100p-noniid-09cpp",
                model_name="orig_cnn",
                R=50,
                E=1,
                C=0.03,
                instance_cores=32,
                timeout=120,
            ),
            VisionTask(
                name="federated-C0_10",
                label="C=0.1",
                dataset_name="fashion-mnist-100p-noniid-09cpp",
                model_name="orig_cnn",
                R=50,
                E=1,
                C=0.1,
                instance_cores=32,
                timeout=120,
            ),
            VisionTask(
                name="federated-C0_20",
                label="C=0.2",
                dataset_name="fashion-mnist-100p-noniid-09cpp",
                model_name="orig_cnn",
                R=50,
                E=1,
                C=0.2,
                instance_cores=32,
                timeout=240,
            ),
        ],
        aggregation_name="flul-aggregation",
    ),
    "C-fashion-mnist-100p-iid-balanced": Benchmark(
        tasks=[
            VisionTask(
                name="federated-C0_01",
                label="C=0.01",
                dataset_name="fashion-mnist-100p-iid-balanced",
                model_name="orig_cnn",
                R=50,
                E=1,
                C=0.01,
                instance_cores=32,
                timeout=120,
            ),
            VisionTask(
                name="federated-C0_03",
                label="C=0.03",
                dataset_name="fashion-mnist-100p-iid-balanced",
                model_name="orig_cnn",
                R=50,
                E=1,
                C=0.03,
                instance_cores=32,
                timeout=120,
            ),
            VisionTask(
                name="federated-C0_10",
                label="C=0.1",
                dataset_name="fashion-mnist-100p-iid-balanced",
                model_name="orig_cnn",
                R=50,
                E=1,
                C=0.1,
                instance_cores=32,
                timeout=120,
            ),
            VisionTask(
                name="federated-C0_20",
                label="C=0.2",
                dataset_name="fashion-mnist-100p-iid-balanced",
                model_name="orig_cnn",
                R=50,
                E=1,
                C=0.2,
                instance_cores=32,
                timeout=240,
            ),
        ],
        aggregation_name="flul-aggregation",
    ),
    #
    # ##############################
    # E LOW-TO-HIGH
    # ##############################
    #
    "E-fashion-mnist-100p-iid-balanced": Benchmark(
        tasks=[
            VisionTask(
                name="federated-E01",
                label="E=01",
                dataset_name="fashion-mnist-100p-iid-balanced",
                model_name="orig_cnn",
                R=128,
                E=1,
                instance_cores=32,
                timeout=120,
            ),
            VisionTask(
                name="federated-E02",
                label="E=02",
                dataset_name="fashion-mnist-100p-iid-balanced",
                model_name="orig_cnn",
                R=64,
                E=2,
                instance_cores=32,
                timeout=120,
            ),
            VisionTask(
                name="federated-E04",
                label="E=04",
                dataset_name="fashion-mnist-100p-iid-balanced",
                model_name="orig_cnn",
                R=32,
                E=4,
                instance_cores=32,
                timeout=120,
            ),
            VisionTask(
                name="federated-E08",
                label="E=08",
                dataset_name="fashion-mnist-100p-iid-balanced",
                model_name="orig_cnn",
                R=16,
                E=8,
                instance_cores=32,
                timeout=120,
            ),
            VisionTask(
                name="federated-E16",
                label="E=16",
                dataset_name="fashion-mnist-100p-iid-balanced",
                model_name="orig_cnn",
                R=8,
                E=16,
                instance_cores=32,
                timeout=120,
            ),
        ],
        aggregation_name="flul-aggregation",
    ),
    "E-fashion-mnist-100p-noniid-08cpp": Benchmark(
        tasks=[
            VisionTask(
                name="federated-E01",
                label="E=01",
                dataset_name="fashion-mnist-100p-noniid-08cpp",
                model_name="orig_cnn",
                R=128,
                E=1,
                instance_cores=32,
                timeout=120,
            ),
            VisionTask(
                name="federated-E02",
                label="E=02",
                dataset_name="fashion-mnist-100p-noniid-08cpp",
                model_name="orig_cnn",
                R=64,
                E=2,
                instance_cores=32,
                timeout=120,
            ),
            VisionTask(
                name="federated-E04",
                label="E=04",
                dataset_name="fashion-mnist-100p-noniid-08cpp",
                model_name="orig_cnn",
                R=32,
                E=4,
                instance_cores=32,
                timeout=120,
            ),
            VisionTask(
                name="federated-E08",
                label="E=08",
                dataset_name="fashion-mnist-100p-noniid-08cpp",
                model_name="orig_cnn",
                R=16,
                E=8,
                instance_cores=32,
                timeout=120,
            ),
            VisionTask(
                name="federated-E16",
                label="E=16",
                dataset_name="fashion-mnist-100p-noniid-08cpp",
                model_name="orig_cnn",
                R=8,
                E=16,
                instance_cores=32,
                timeout=120,
            ),
        ],
        aggregation_name="flul-aggregation",
    ),
    "E-fashion-mnist-100p-noniid-05cpp": Benchmark(
        tasks=[
            VisionTask(
                name="federated-E01",
                label="E=01",
                dataset_name="fashion-mnist-100p-noniid-05cpp",
                model_name="orig_cnn",
                R=128,
                E=1,
                instance_cores=32,
                timeout=120,
            ),
            VisionTask(
                name="federated-E02",
                label="E=02",
                dataset_name="fashion-mnist-100p-noniid-05cpp",
                model_name="orig_cnn",
                R=64,
                E=2,
                instance_cores=32,
                timeout=120,
            ),
            VisionTask(
                name="federated-E04",
                label="E=04",
                dataset_name="fashion-mnist-100p-noniid-05cpp",
                model_name="orig_cnn",
                R=32,
                E=4,
                instance_cores=32,
                timeout=120,
            ),
            VisionTask(
                name="federated-E08",
                label="E=08",
                dataset_name="fashion-mnist-100p-noniid-05cpp",
                model_name="orig_cnn",
                R=16,
                E=8,
                instance_cores=32,
                timeout=120,
            ),
            VisionTask(
                name="federated-E16",
                label="E=16",
                dataset_name="fashion-mnist-100p-noniid-05cpp",
                model_name="orig_cnn",
                R=8,
                E=16,
                instance_cores=32,
                timeout=120,
            ),
        ],
        aggregation_name="flul-aggregation",
    ),
    #
    # ##############################
    # End-To-End Tests
    # ##############################
    #
    "e2e-flul-fashion-mnist": Benchmark(
        tasks=[
            UnitaryVisionTask(
                name="unitary",
                dataset_name="fashion-mnist-100p-noniid-02cpp",
                E=4,
                partition_id=0,
                instance_cores=4,
                timeout=10,
            ),
            VisionTask(
                name="federated",
                dataset_name="fashion-mnist-100p-noniid-02cpp",
                R=2,
                E=2,
                C=0.02,
                instance_cores=4,
                timeout=10,
            ),
        ],
        aggregation_name="flul-aggregation",
    ),
    "e2e-flul-cifar-10": Benchmark(
        tasks=[
            UnitaryVisionTask(
                name="unitary",
                dataset_name="cifar-10-100p-noniid-02cpp",
                E=4,
                partition_id=0,
                instance_cores=16,
                timeout=10,
            ),
            VisionTask(
                name="federated",
                dataset_name="cifar-10-100p-noniid-02cpp",
                R=2,
                E=2,
                C=0.02,
                instance_cores=16,
                timeout=10,
            ),
        ],
        aggregation_name="flul-aggregation",
    ),
}


def run_benchmark(benchmark_name: str):
    logging.info(f"Building Docker image for benchmark {benchmark_name}")

    logging.info(f"Starting benchmark {benchmark_name}")
    benchmark = benchmarks[benchmark_name]

    group_name = FLAGS.group_name or f"{strftime('%Y%m%dT%H%M')}_{benchmark_name}"

    task_names = {task.name for task in benchmark.tasks}

    assert len(task_names) == len(benchmark.tasks), "Duplicate task names"

    should_push = benchmark.runner == "ec2"
    docker_image_name = docker.build(should_push=should_push)

    # TODO Initiate tasks in parallel
    for task in benchmark.tasks:
        model_name = task.model_name
        dataset_name = task.dataset_name
        run_task(
            docker_image_name=docker_image_name,
            group_name=group_name,
            task_name=task.name,
            task_label=task.label,
            model=model_name,
            dataset=dataset_name,
            R=task.R,
            E=task.E,
            C=task.C,
            B=task.B,
            partition_id=task.partition_id,
            instance_cores=task.instance_cores,
            timeout=task.timeout,
            runner=benchmark.runner,
        )

    with TemporaryDirectory() as tmpdir:
        fname = os.path.join(tmpdir, "config.json")
        data = {"aggregation_name": benchmark.aggregation_name}
        storage.write_json(data, fname)
        results.push(group_name=group_name, task_name="", output_dir=tmpdir)


def run_task(
    docker_image_name: str,
    group_name: str,
    task_name: str,
    task_label: str,
    model: str,
    dataset: str,
    R: int,
    E: int,
    C: float,
    B: int,
    partition_id: Optional[int],
    instance_cores: int,
    timeout: int,
    runner: str,  # one of ["ec2", "docker"]
):
    task_msg = f"{model}, {dataset}, {R}, {E}, {C}, {B}, {instance_cores}, {timeout}"
    logging.info(f"Attempting to run task on {runner}: {task_msg}")

    if runner == "ec2":
        r = run.ec2
    elif runner == "docker":
        r = run.docker
    else:
        raise Exception("Runner does not exist")

    r(
        image=docker_image_name,
        timeout=timeout,
        instance_cores=instance_cores,
        # The following arguments will be passed as absl flags:
        group_name=group_name,
        task_name=task_name,
        task_label=task_label,
        model=model,
        dataset=dataset,
        R=R,
        E=E,
        C=C,
        B=B,
        partition_id=partition_id,
    )


def main(_):
    benchmark_name = FLAGS.benchmark_name
    assert benchmark_name in benchmarks
    run_benchmark(benchmark_name=benchmark_name)
