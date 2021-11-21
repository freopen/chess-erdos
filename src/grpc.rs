use anyhow::{Context, Result};
use pbdb::Collection;
use tonic::{Request, Response, Status};

use crate::{
  proto::{
    chess_erdos_service_server::ChessErdosService, ErdosChain, ErdosLink, GetErdosChainsRequest,
    GetErdosChainsResponse, User,
  },
  util::user_to_erdos_number,
};

#[derive(Default)]
pub struct ChessErdosServiceImpl {}

fn expand_erdos_chain(erdos_link: ErdosLink) -> Result<ErdosChain> {
  let mut erdos_links = vec![erdos_link];
  for erdos_number in (1..erdos_links[0].erdos_number).rev() {
    let next_user =
      User::get(&erdos_links.last().unwrap().loser_id)?.context("Broken chain in DB")?;
    let next_erdos_link = next_user
      .erdos_links
      .into_iter()
      .find(|erdos_link| erdos_link.erdos_number == erdos_number)
      .context("Broken chain in DB")?;
    erdos_links.push(next_erdos_link);
  }
  Ok(ErdosChain { erdos_links })
}

fn build_erdos_chains(user: User) -> Result<GetErdosChainsResponse> {
  Ok(GetErdosChainsResponse {
    id: user.id.clone(),
    erdos_number: user_to_erdos_number(&user),
    erdos_chains: user
      .erdos_links
      .into_iter()
      .map(expand_erdos_chain)
      .collect::<Result<Vec<_>>>()?,
  })
}

#[tonic::async_trait]
impl ChessErdosService for ChessErdosServiceImpl {
  async fn get_erdos_chains(
    &self,
    request: Request<GetErdosChainsRequest>,
  ) -> Result<Response<GetErdosChainsResponse>, Status> {
    match User::get(&request.into_inner().id) {
      Err(err) => Err(Status::internal(err.to_string())),
      Ok(None) => Err(Status::not_found("User not found")),
      Ok(Some(user)) => match build_erdos_chains(user) {
        Err(err) => Err(Status::internal(err.to_string())),
        Ok(erdos_chains) => Ok(Response::new(erdos_chains)),
      },
    }
  }
}
