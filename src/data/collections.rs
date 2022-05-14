use rkyvdb::{Collection, CaseInsensitiveString};

use super::{User, ServerMetadata};

impl Collection for User {
    type KeyType = CaseInsensitiveString;
    const CF_NAME: &'static str = "users";
}

impl Collection for ServerMetadata {
    type KeyType = ();
    const CF_NAME: &'static str = "metadata";
}
