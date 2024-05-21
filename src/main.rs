#![allow(non_snake_case)]

use dioxus::prelude::*;
use tracing::Level;

use pages::*;

#[cfg(feature = "server")]
mod helpers;
mod pages;
mod rgb;

#[derive(Clone, Routable, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Route {
    #[route("/")]
    Home {},
    #[route("/increase_contrast")]
    IncreaseContrast {},
    #[route("/excel_from_image")]
    ExcelFromImage {},
    #[route("/get_pixel_color")]
    GetPixelColor {},
    #[route("/replace_color")]
    ReplaceColor {},
    #[route("/rgb_to_grayscale")]
    RgbToGrayscale {},
    #[route("/excel_to_image")]
    ExcelToImage {},
    #[route("/rgb_to_cmyk")]
    RgbToCmyk {},
    #[route("/blur")]
    Blur {},
    #[route("/edge_detection")]
    EdgeDetection {},
    #[route("/remove_color")]
    RemoveColor {}
}

fn main() {
    // Init logger
    dioxus_logger::init(Level::INFO).expect("failed to init logger");
    launch(App);
}

fn App() -> Element {
    rsx! {
        Router::<Route> {}
    }
}

#[component]
fn Home() -> Element {

    rsx! {
        div {
            ul {
                li {
                    Link {
                        to: Route::IncreaseContrast {}, "Increase image contrast"
                    }
                }
                li {
                    Link {
                        to: Route::ExcelFromImage {}, "Excel from image"
                    }
                }
                li {
                    Link {
                        to: Route::ExcelToImage {}, "Excel to image"
                    }
                }
                li {
                    Link {
                        to: Route::GetPixelColor {}, "Get pixel color"
                    }
                }
                li {
                    Link {
                        to: Route::ReplaceColor {}, "Replace pixel color"
                    } 
                }
                li {
                    Link {
                        to: Route::RgbToGrayscale {}, "Rgb image to grayscale"
                    } 
                }
                li {
                    Link {
                        to: Route::RgbToCmyk {}, "Rgb image to CMYK"
                    } 
                }
                li {
                    Link {
                        to: Route::Blur {}, "Blur image"
                    } 
                }
                li {
                    Link {
                        to: Route::EdgeDetection {}, "Edge detection"
                    } 
                }
                li {
                    Link {
                        to: Route::RemoveColor {}, "Remove color"
                    } 
                }
            }
        }
    }
}


#[server(GetServerData)]
async fn get_server_data() -> Result<String, ServerFnError> {
    Ok("Hello from the server!".to_string())
}
