use std::{
  env,
  path::{Path, PathBuf},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
  pbdb::create_pbdb_proto(Path::new("proto"));
  tonic_build::configure()
    .build_client(false)
    .file_descriptor_set_path(
      PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR environment variable not set"))
        .join("file_descriptor_set.bin"),
    )
    .compile(&["proto/chess_erdos.proto"], &["proto"])?;
  Ok(())
}
