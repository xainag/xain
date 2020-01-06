[![CircleCI](https://img.shields.io/circleci/build/github/xainag/xain-fl/master?style=flat-square)](https://circleci.com/gh/xainag/xain-fl/tree/master)
[![PyPI](https://img.shields.io/pypi/v/xain-fl?style=flat-square)](https://pypi.org/project/xain-fl/)
[![GitHub license](https://img.shields.io/github/license/xainag/xain-fl?style=flat-square)](https://github.com/xainag/xain-fl/blob/master/LICENSE)
[![Documentation Status](https://readthedocs.org/projects/xain-fl/badge/?version=latest)](https://xain-fl.readthedocs.io/en/latest/?badge=latest)
[![Gitter chat](https://badges.gitter.im/xainag.png)](https://gitter.im/xainag)

# XAIN

The XAIN project is building a privacy layer for machine learning so that AI projects can meet compliance such as
GDPR and CCPA. The approach relies on Federated Learning as enabling technology that allows production AI
applications to be fully privacy compliant.

Federated Learning also enables different use-cases that are not strictly privacy related such as connecting data 
lakes, reaching higher model performance in unbalanced datasets and utilising AI models on the edge.

This repository contains the source code for running the Coordinator. The Coordinator is the component of Federated
Learning that selects the Participants for training and aggregates the models using federated averaging.

The Participants run in a separate environment than the Coordinator and connect to it using an SDK. You can find [here](https://github.com/xainag/xain-sdk) the source code for it.

## Quick Start

XAIN requires [Python 3.6.4+](https://python.org/). To install the `xain-fl` package just run:

```shell
$ python -m pip install xain-fl
```

## Install from source

To clone this repository and to install the XAIN-FL project, please execute the following commands:

```shell
$ git clone https://github.com/xainag/xain-fl.git
$ cd xain-fl

$ sh scripts/setup.sh
```

### Verify Installation

You can verify the installation by running the tests

```shell
$ pytest
```

### Building the Documentation

The project documentation resides under `docs/`. To build the documentation
run:

```shell
$ cd docs/
$ make docs
```

The generated documentation will be under `docs/_build/html/`. You can open the
root of the documentation by opening `docs/_build/html/index.html` on your
favorite browser or simply run the command:

```shell
$ make show
```

### Running the Coordinator locally

To run the Coordinator on your local machine, use the command:

```shell
$ python xain_fl/cli.py -f test_array.npy
```

For more information about the CLI and its arguments, run:

```shell
$ python xain_fl/cli.py --help
```

### Run the Coordinator from a Docker image

Development image
---

To run the coordinator's development image, first build the Docker image:

```shell
$ docker build -t xain-fl-dev -f dev.dockerfile .
```

Then run the image, mounting the directory as a Docker volume, and call the
entrypoint:

```shell
$ docker run -v $(pwd):/app -v '/app/xain_fl.egg-info' xain-fl-dev coordinator
```

Release image
---

To run the coordinator's release image, first build it:

```shell
$ docker build -t xain-fl .
```

And then run it (this example assumes you'll want to use the default port):

```shell
$ docker run -p 50051:50051 xain-fl
```

## Related Papers and Articles

- [An introduction to XAIN’s GDPR-compliance Layer for Machine Learning](https://medium.com/xain/an-introduction-to-xains-gdpr-compliance-layer-for-machine-learning-f7c321b31b06)
- [Communication-Efficient Learning of Deep Networks from Decentralized Data](https://arxiv.org/abs/1602.05629)
- [Analyzing Federated Learning through an Adversarial Lens](https://arxiv.org/abs/1811.12470)
- [Towards Federated Learning at Scale: System Design](https://arxiv.org/abs/1902.01046)
