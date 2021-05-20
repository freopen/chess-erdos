use anyhow::Result;
use heed::RoTxn;
use tonic::{transport::Server, Status};
use tonic::{Request, Response};

use crate::{
    db::{ENV, USERS},
    proto::{
        chess_erdos_service_server::{ChessErdosService, ChessErdosServiceServer},
        ErdosChain, ErdosLink, GetErdosLinksRequest, GetErdosLinksResponse, User,
    },
};

fn build_erdos_chain(txn: &mut RoTxn, erdos_link: ErdosLink) -> ErdosChain {
    let mut chain = ErdosChain::default();
    let mut cur_user_id = erdos_link.loser_id.to_ascii_lowercase();
    let mut cur_erdos_number = erdos_link.erdos_number - 1;
    chain.erdos_links.push(erdos_link);
    while cur_erdos_number > 0 {
        let user: User = USERS.get(txn, &cur_user_id).unwrap().unwrap();
        let next_erdos_link = user
            .erdos_links
            .into_iter()
            .find(|erdos_link| erdos_link.erdos_number == cur_erdos_number)
            .unwrap();
        cur_user_id = next_erdos_link.loser_id.to_ascii_lowercase();
        cur_erdos_number -= 1;
        chain.erdos_links.push(next_erdos_link);
    }
    chain
}

struct ChessErdosServiceImpl {}

#[tonic::async_trait]
impl ChessErdosService for ChessErdosServiceImpl {
    async fn get_erdos_links(
        &self,
        request: Request<GetErdosLinksRequest>,
    ) -> Result<Response<GetErdosLinksResponse>, Status> {
        let mut txn = ENV.read_txn().unwrap();
        let user: Option<User> = USERS
            .get(&mut txn, &request.get_ref().user_id.to_ascii_lowercase())
            .unwrap();
        if let Some(user) = user {
            Ok(Response::new(GetErdosLinksResponse {
                user_id: user.id,
                erdos_chains: user
                    .erdos_links
                    .into_iter()
                    .rev()
                    .map(|erdos_link| build_erdos_chain(&mut txn, erdos_link))
                    .collect(),
            }))
        } else {
            Err(Status::not_found("User not found"))
        }
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
