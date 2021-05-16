use anyhow::Result;
use tonic::{transport::Server, Status};

use crate::proto::{GetErdosLinksRequest, GetErdosLinksResponse, chess_erdos_service_server::{ChessErdosService, ChessErdosServiceServer}};

struct ChessErdosServiceImpl {}

#[tonic::async_trait]
impl ChessErdosService for ChessErdosServiceImpl {
    async fn get_erdos_links(
        &self,
        _request: tonic::Request<GetErdosLinksRequest>,
    ) -> Result<tonic::Response<GetErdosLinksResponse>, Status> {
        todo!()
    }
}

pub async fn tonic_server_task() -> Result<()> {
    let addr = "127.0.0.1:50051".parse().unwrap();
    let service = ChessErdosServiceImpl {};
    Server::builder()
        .add_service(ChessErdosServiceServer::new(service))
        .serve(addr)
        .await?;
    Ok(())
}
