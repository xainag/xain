"""This module provides helpers for reading and validating the TOML
configuration.

"""

# Implementation notes:
# =====================
#
# We use the `schema` library to validate the configuration provided
# by the user. However, `schema` works mainly with dictionaries, which
# are not particularly convenient for us:
#
# - we cannot generate documentation for all the dictionary keys
# - the syntax for accessing values is quite verbose, especially if
#   the dictionary has nested values
#
# Thus, although we use schemas to validate configurations (AI_SCHEMA,
# SERVER_SCHEMA, and STORAGE_SCHEMA), we don't expose them. Instead,
# we use them to dynamically generate classes where attributes are the
# schema keys: AiConfig, ServerConfig, and StorageConfig. This
# hackery happens in create_class_from_schema(). It is not
# perfect. For instance, we cannot document the type of each
# attribute. But it is arguably better than using plain dictionaries.

from collections import namedtuple
import ipaddress
import os
from typing import Any, Mapping, NamedTuple, Type, TypeVar
import urllib

import idna
from schema import And, Optional, Or, Schema, SchemaError, Use
import toml

from xain_fl.logger import StructLogger, get_logger

logger: StructLogger = get_logger(__name__)


def error(key: str, description: str) -> str:
    """Return an error message for the given configuration item and
    description of the expected value type.

    Args:

        key (str): key of the configuration item
        description (str): description of the expected type of value
            for this configuration item
    """
    return f"Invalid `{key}`: value must be {description}"


def positive_integer(key: str, expected_value: str = "a strictly positive integer") -> Schema:
    """Return a validator for strictly positive integers for the given
    configuration item.

    Args:

        key (str): key of the configuration item
        expected_value (str): description of the expected type of
            value for this configuration item
    """
    return And(int, lambda value: value > 0, error=error(key, expected_value))


def non_negative_integer(key: str, expected_value: str = "a positive integer") -> Schema:
    """Return a non-negative integer validator for the given configuration
    item.

    Args:

        key: key of the configuration item
        expected_value: description of the expected type of
            value for this configuration item

    """
    return And(int, lambda value: value >= 0, error=error(key, expected_value))


def url(key: str, expected_value: str = "a valid URL") -> Schema:
    """Return a URL validator for the given configuration item.

    Args:

        key: key of the configuration item
        expected_value: description of the expected type of
            value for this configuration item

    """

    def is_valid_url(value):
        try:
            parsed = urllib.parse.urlparse(value)
        except (ValueError, urllib.error.URLError):
            return False
        # A URL is considered valid if it has at least a scheme and a
        # network location.
        return all([parsed.scheme, parsed.netloc])

    return And(str, is_valid_url, error=error(key, expected_value))


def is_valid_hostname(value: str) -> bool:
    """Return whether the given string is a valid hostname

    Args:

        value: string to check

    Returns:

        `True` if the given value is a valid hostname, `False`
        otherwise
    """
    try:
        idna.encode(value)
    except idna.IDNAError:
        return False
    return True


def is_valid_ip_address(value: str) -> bool:
    """Return whether the given string is a valid IP address

    Args:

        value: string to check

    Returns:

        `True` if the given value is a valid IP address, `False`
        otherwise
    """
    try:
        ipaddress.ip_address(value)
    except ipaddress.AddressValueError:
        return False
    return True


def hostname_or_ip_address(
    key: str, expected_value: str = "a valid domain name or IP address"
) -> Schema:
    """Return a hostname or IP address validator for the given
    configuration item.

    Args:

        key: key of the configuration item
        expected_value: description of the expected type of
            value for this configuration item

    """
    return And(str, Or(is_valid_hostname, is_valid_ip_address), error=error(key, expected_value),)


def abs_path(value: str) -> str:
    """
    If the given value is a relative path, return the corresponding absolute path.

    Args:

        value: a path
    """
    if not os.path.isabs(value):
        path = os.path.abspath(value)
        logger.warn(
            "relative path will be converted to an absolute path", path=value, absolute_path=path
        )
        return path
    return value


def existing_file(key: str, expected_value: str = "a valid path to an existing file") -> Schema:
    """Return a validator for paths to an existing file for the given
    configuration item.

    Args:

        key: key of the configuration item
        expected_value: description of the expected type of
            value for this configuration item

    """

    def validate_path(path):
        path = abs_path(path)
        if not os.path.isfile(path):
            logger.error("File not found", path=path)
            return False
        return True

    return And(str, validate_path, error=error(key, expected_value))


SERVER_SCHEMA = Schema(
    {
        Optional("host", default="localhost"): hostname_or_ip_address("server.host"),
        Optional("port", default=50051): Use(
            int, error=error("server.port", "a valid port number")
        ),
    }
)

AI_SCHEMA = Schema(
    {
        "rounds": positive_integer("ai.rounds"),
        "epochs": non_negative_integer("ai.epochs"),
        Optional("min_participants", default=1): positive_integer("ai.min_participants"),
        Optional("fraction_participants", default=1.0): And(
            Or(int, float),
            lambda value: 0 < value <= 1.0,
            error=error("ai.fraction_participants", "a float between 0 and 1.0"),
        ),
    }
)

STORAGE_SCHEMA = Schema(
    {
        "endpoint": And(str, url, error=error("storage.endpoint", "a valid URL")),
        "bucket": Use(str, error=error("storage.endpoint", "an S3 bucket name")),
        "secret_access_key": Use(
            str, error=error("storage.secret_access_key", "a valid utf-8 string")
        ),
        "access_key_id": Use(str, error=error("storage.access_key_id", "a valid utf-8 string")),
    }
)

CONFIG_SCHEMA = Schema(
    {
        Optional("server", default=SERVER_SCHEMA.validate({})): SERVER_SCHEMA,
        "ai": AI_SCHEMA,
        "storage": STORAGE_SCHEMA,
    }
)


def create_class_from_schema(class_name: str, schema: Schema) -> Any:

    """Create a class named `class_name` from the given `schema`, where
    the attributes of the new class are the schema's keys.

    Args:

        class_name: name of the class to create
        schema: schema from which to create the class

    Returns:

        A new class where attributes are the given schema's keys
    """
    # pylint: disable=protected-access
    keys = schema._schema.keys()
    attributes = list(map(lambda key: key._schema if isinstance(key, Schema) else key, keys))
    return namedtuple(class_name, attributes)


# pylint: disable=invalid-name
AiConfig = create_class_from_schema("AiConfig", AI_SCHEMA)
AiConfig.__doc__ = "FL configuration: number of participant to each training round, etc."

ServerConfig = create_class_from_schema("ServerConfig", SERVER_SCHEMA)
ServerConfig.__doc__ = "The server configuration: TLS, addresses for incoming connections, etc."

StorageConfig = create_class_from_schema("StorageConfig", STORAGE_SCHEMA)
StorageConfig.__doc__ = "Storage related configuration: storage endpoints and credentials, etc."

T = TypeVar("T", bound="Config")


class Config:
    """The coordinator configuration.

    Configuration is split in three sections: `Config.ai` for items
    directly related to the FL protocol, `Config.server` for the
    server infrastructure, and `Config.storage` for storage related
    items.

    The configuration is usually loaded from a dictionary the `Config`
    attributes map to the dictionary keys.

    Args:

        ai: the configuration corresponding to the `[ai]` section of
            the toml config file

        storage: the configuration corresponding to the `[storage]`
            section of the toml config fil

        server: the configuration corresponding to the `[server]`
            section of the toml config file

    :Example:

    Here is a valid configuration:

    .. code-block:: toml

       # This section correspond to the `Config.server` attribute
       [server]

       # Address to listen on for incoming gRPC connections
       host = "[::]"
       # Port to listen on for incoming gRPC connections
       port = 50051


       # This section corresponds to the `Config.ai` attribute
       [ai]

       # Number of global rounds the model is going to be trained for. This
       # must be a positive integer.
       rounds = 1

       # Number of local epochs per round
       epochs = 1

       # Minimum number of participants to be selected for a round.
       min_participants = 1

       # Fraction of total clients that participate in a training round. This
       # must be a float between 0 and 1.
       fraction_participants = 1.0

       # This section corresponds to the `Config.storage` attribute
       [storage]

       # URL to the storage service to use
       endpoint = "http://localhost:9000"

       # Name of the bucket for storing the aggregated models
       bucket = "aggregated_weights"

       # AWS secret access to use to authenticate to the storage service
       secret_access_key = "my-secret"

       # AWS access key ID to use to authenticate to the storage service
       access_key_id = "my-key-id"

    This config file can be loaded and used as follows:

    .. code-block:: python

       from xain_fl.config import Config

       config = Config.load("example_config.toml")

       assert config.server.host == "[::]"
       assert config.server.port == 50051

       assert config.ai.rounds == 1
       assert config.ai.epochs == 1
       assert config.ai.min_participants == 1
       assert config.ai.fraction_participants == 1.0

       assert config.storage.endpoint == "http://localhost:9000"
       assert config.storage.bucket == "aggregated_weights"
       assert config.storage.secret_access_key == "my-access-key"
       assert config.storage.access_key_id == "my-key"
    """

    def __init__(self, ai: NamedTuple, storage: NamedTuple, server: NamedTuple):
        self.ai = ai
        self.storage = storage
        self.server = server

    @classmethod
    def from_unchecked_dict(cls: Type[T], dictionary: Mapping[str, Any]) -> T:
        """Check if the given dictionary contains a valid configuration, and
        if so, create a `Config` instance from it.

        Args:

            dictionary: a dictionary containing the configuration
        """
        try:
            valid_config = CONFIG_SCHEMA.validate(dictionary)
        except SchemaError as err:
            raise InvalidConfig(err.code) from err
        return cls.from_valid_dict(valid_config)

    @classmethod
    def from_valid_dict(cls: Type[T], dictionary: Mapping[str, Any]) -> T:
        """Create a `Config` instance for the given dictionary, assuming it
        contains a valid configuration

        Args:

            dictionary: a dictionary containing the configuration

        """
        return cls(
            AiConfig(**dictionary["ai"]),
            StorageConfig(**dictionary["storage"]),
            ServerConfig(**dictionary["server"]),
        )

    @classmethod
    def load(cls: Type[T], path: str) -> T:
        """Read the config file from the given path, check that it contains a
        valid configuration, and return the corresponding `Config`
        instance.

        Args:

            path: path to a configuration file
        """
        try:
            with open(path, "r") as f:
                raw_config = toml.load(f)
        except IsADirectoryError as err:
            raise InvalidConfig(f"{path} is a directory") from err
        except FileNotFoundError as err:
            raise InvalidConfig(f"{path} not found") from err
        except PermissionError as err:
            raise InvalidConfig(f"failed to read {path}: insufficient permissions") from err
        except toml.TomlDecodeError as err:
            raise InvalidConfig(f"failed to decode {path}: {err}") from err
        except OSError as err:
            raise InvalidConfig(str(err)) from err
        return cls.from_unchecked_dict(raw_config)


class InvalidConfig(ValueError):
    """
    Exception raised upon trying to load an invalid configuration
    """
