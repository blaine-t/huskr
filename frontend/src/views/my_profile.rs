use crate::components::Hero;
use dioxus::prelude::*;
use serde::Deserialize;
use super::API_URL;

/// The Home page component that will be rendered when the current route is `[Route::Home]`
#[component]
pub fn MyProfile() -> Element {
    let profile = use_resource(move || async move {
        retrieve_user_self().await
    });

    rsx! {
        div {
            if let Some(profile) = &*profile.read() {
                match profile {
                    Ok(_) => {
                       rsx!{
                        div {
                            h1 { "we did it" }
                        }
                        
                    }
                    },
                    Err(e) => {
                       rsx!{
                        div {
                            h1 { "{e:?}" }
                        }
                        }
                    },
                }
            } else {
                h1 { "loading..." },
            },
         }
    }   
}

async fn retrieve_user_self() -> Result<UserResponse, reqwest::Error> {
    let api_url = format!("{API_URL}user/me");
    let client = reqwest::Client::new();
    let mut response = client.get(api_url)
    .fetch_credentials_include()
    .header("access-control-allow-credentials", "include")
    .send().await?;

    return response.json::<UserResponse>().await;
}

/// Public-facing user representation sent to the frontend.
/// Deliberately omits all OAuth tokens.
#[derive(Debug, Deserialize, Clone)]
pub struct UserResponse {
    pub id: i64,
    pub oid: String,
    pub email: Option<String>,
    pub display_name: Option<String>,
    pub tenant_id: Option<String>,
    // Profile fields
    pub full_name: Option<String>,
    pub age: Option<i64>,
    pub is_rso: bool,
    pub major: Option<String>,
    pub bio: Option<String>,
    pub image_key: Option<String>,
    pub interests: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}
