#![allow(non_snake_case)]
#![allow(unused_imports)]

use dioxus::prelude::*;
use gloo::file::{Blob, File, ObjectUrl};

use tracing::info;

use crate::Route;

#[component]
pub fn GetPixelColor() -> Element {
    // Urls are relative to your Cargo.toml file

    let mut input_file = use_signal(|| File::new("input_file.jpg", Vec::<u8>::new().as_slice()));
    let mut x = use_signal(|| 0);
    let mut y = use_signal(|| 0);
    let mut pixel_color: Signal<Option<(u8, u8, u8)>> = use_signal(|| None);

    rsx! {
        div {
            style {{include_str!("./get_pixel_color/style.css")}}
            Link {to: Route::Home {}, "Back"}
            br{}
            form {
                onsubmit: move |_| {
                    async move {
                        let data = gloo::file::futures::read_as_bytes(&input_file()).await.unwrap();
                        let resp = get_pixel_color(data, (x(), y())).await.unwrap();
                        pixel_color.set(Some(resp));

                    }
                },
                label {
                    r#for: "input-file",

                    "Select an image"
                }
                br{}
                input {
                    required: true,
                    r#type: "file",
                    accept: ".jpg, .jpeg",
                    name: "input-file",
                    onchange: move |ev| {
                        async move {
                            if let Some(fe) = ev.files() {
                                let files = fe.files();
                                let f = files.first().unwrap();

                                if let Some(f) = fe.read_file(f).await
                                {
                                    input_file.set(File::new("input_file.jpg", f.as_slice()));
                                }
                            }
                        }
                    },

                }
                br{}
                br{}
                label {
                    r#for: "ammount-to-increase",

                    "X & Y: "
                }
                br{}
                input {
                    r#type: "number",
                    name: "ammount-to-increase",
                    onchange: move |ev| {
                        x.set(ev.value().parse().unwrap());
                    }
                }
                input {
                    r#type: "number",
                    name: "ammount-to-increase",
                    onchange: move |ev| {
                        y.set(ev.value().parse().unwrap());
                    }
                }
                br{}br{}
                button {
                    r#type: "submit",

                    "Submit"
                }
            }
            div {
                if let Some((r, g, b)) = pixel_color() {
                    div {
                        class: "pixel",
                        style: format!("background-color: rgb({r}, {g}, {b})"),

                        {format!("The color of the pixel is: ")}
                        br{}
                        {format!("R: {r}; G: {g}; B: {b}")} 
                    }
                }
            }
        }
    }
}

#[server]
async fn get_pixel_color(data: Vec<u8>, pixel: (u32, u32)) -> Result<(u8, u8, u8), ServerFnError> {
    use tempfile::NamedTempFile;
    use image::{self, Pixel};
    use image::GenericImageView;
    use image::DynamicImage;

    let img = image::load_from_memory(data.as_slice()).unwrap();
    let pixel = img.get_pixel(pixel.1, pixel.0);
    let rgb = (pixel.0[0], pixel.0[1], pixel.0[2]);
    Ok(rgb)
}
