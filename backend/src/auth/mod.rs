pub mod backend;
pub mod routes;

pub use backend::MicrosoftBackend;

#[derive(Clone, Debug)]
pub struct Credentials {
    pub code: String,
    pub pkce_verifier: String,
}
