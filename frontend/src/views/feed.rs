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
                    img { src: "https://www.shutterstock.com/image-photo/fashion-industry-black-woman-designer-600nw-2235667567.jpg" },
                    h1 { 
                        class: "top_left",
                        "Rachel"
                    }
                }
                h2 { "Hi! I'm Rachel. I'm 15 and a math major. Looking for fellow travelers to Little St. James."}
                h1 {
                    "Marry?"
                }
                div { 
                    class: "stacked",
                    button { 
                        style: "background-color: var(--accept-color)",
                        class: "user_button",
                        "YES PLEASE"
                    }
                    button { 
                        style: "background-color: var(--deny-color)",
                        class: "user_button",
                        "TOO YOUNG"
                    }
                }

                
                
            }
            div {

            }
        }
    }
}
