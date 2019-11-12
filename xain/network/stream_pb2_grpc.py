# Generated by the gRPC Python protocol compiler plugin. DO NOT EDIT!
import grpc

from xain.network import stream_pb2 as xain_dot_network_dot_stream__pb2


class ParticipantManagerStub(object):
  # missing associated documentation comment in .proto file
  pass

  def __init__(self, channel):
    """Constructor.

    Args:
      channel: A grpc.Channel.
    """
    self.Connect = channel.stream_stream(
        '/ParticipantManager/Connect',
        request_serializer=xain_dot_network_dot_stream__pb2.ParticipantMessage.SerializeToString,
        response_deserializer=xain_dot_network_dot_stream__pb2.CoordinatorMessage.FromString,
        )


class ParticipantManagerServicer(object):
  # missing associated documentation comment in .proto file
  pass

  def Connect(self, request_iterator, context):
    # missing associated documentation comment in .proto file
    pass
    context.set_code(grpc.StatusCode.UNIMPLEMENTED)
    context.set_details('Method not implemented!')
    raise NotImplementedError('Method not implemented!')


def add_ParticipantManagerServicer_to_server(servicer, server):
  rpc_method_handlers = {
      'Connect': grpc.stream_stream_rpc_method_handler(
          servicer.Connect,
          request_deserializer=xain_dot_network_dot_stream__pb2.ParticipantMessage.FromString,
          response_serializer=xain_dot_network_dot_stream__pb2.CoordinatorMessage.SerializeToString,
      ),
  }
  generic_handler = grpc.method_handlers_generic_handler(
      'ParticipantManager', rpc_method_handlers)
  server.add_generic_rpc_handlers((generic_handler,))
