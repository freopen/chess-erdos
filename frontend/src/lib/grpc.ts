import { GrpcWebFetchTransport } from '@protobuf-ts/grpcweb-transport';
import { ChessErdosServiceClient } from '../proto/service.client';

const transport = new GrpcWebFetchTransport({
  baseUrl: 'http://localhost:8080',
});
const grpc = new ChessErdosServiceClient(transport);

export default grpc;
