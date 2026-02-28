use crate::components::Hero;
use dioxus::{html::{img::src}, prelude::*};

/// The Home page component that will be rendered when the current route is `[Route::Home]`
#[component]
pub fn Feed() -> Element {
    rsx! {
        div {
            class: "page_wrapper",
            div {

            }
            div {
                class: "card",
                div {
                    class: "image_container",
                    img { src: "https://www.ciee.org/sites/default/files/styles/650h/public/blog/2018-05/6a010536fa9ded970b0224df30ce71200b.jpg?itok=iJzB8XaM" },
                    h1 { 
                        class: "top_left",
                        "Rachel"
                    }
                }
                
                h2 { "Hi! I'm Rachel. I'm 15 and a math major. Looking for fellow travelers to Little St. James."}
                button { 
                    class: "user_button",
                    "Match?"
                 }
                 
            }
            div {

            }
        }
    }
}
