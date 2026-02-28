use crate::Route;
use dioxus::prelude::*;

const NAVBAR_CSS: Asset = asset!("/assets/styling/navbar.css");

const LOGO: Asset = asset!("/assets/logo.png");


/// The Navbar component that will be rendered on all pages of our app since every page is under the layout.
///
///
/// This layout component wraps the UI of [Route::Home] and [Route::Blog] in a common navbar. The contents of the Home and Blog
/// routes will be rendered under the outlet inside this component
#[component]
pub fn Navbar() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: NAVBAR_CSS }
        document::Link { rel: "stylesheet", href: "https://fonts.googleapis.com/css2?family=Roboto+Condensed:ital,wght@0,100..900;1,100..900&family=Work+Sans:ital,wght@0,100..900;1,100..900&display=swap"}
        div {
            class: "navbar",
            id: "navbar",
            ul {  
                li {
                    float: "left",
                    h1 {  
                    Link {
                        to: Route::Home {},
                        "HUSKR",
                    }
                    }
                }
            
            li {
                float: "right",
                h1 {
                    Link {
                    to: Route::Register {  },
                    "Register"
                }
                }
                
            }
            li {
                float: "right",
                h1 {  
                Link {
                to: Route::Feed {},
                "Feed"
                }
                }
            }
            
            
            }
        }

        // The `Outlet` component is used to render the next component inside the layout. In this case, it will render either
        // the [`Home`] or [`Blog`] component depending on the current route.
        Outlet::<Route> {}
    }
}
