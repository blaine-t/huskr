use crate::components::Hero;
use dioxus::prelude::*;

/// The Home page component that will be rendered when the current route is `[Route::Home]`
#[component]
pub fn Register() -> Element {
    rsx! {
        
        div {
            class: "page_wrapper",
            div {

            }
            div {
                h1 {"Sign up with UNL SSO:"}
                a {
                    href: "https://api.huskr.us/auth/login", "ligma",
                }
                
            }
            div {

            }
        }

    }
}