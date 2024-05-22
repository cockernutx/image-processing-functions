#![allow(non_snake_case)]
#![allow(unused_imports)]

use dioxus::prelude::*;
use gloo::file::{Blob, File, ObjectUrl};

use tracing::info;

use crate::Route;

#[component]
pub fn RemoveColor() -> Element {
    let mut input_file = use_signal(|| File::new("input_file.jpg", Vec::<u8>::new().as_slice()));
    let mut output_file: Signal<Option<ObjectUrl>> = use_signal(|| None);
    let mut r = use_signal(|| true);
    let mut g = use_signal(|| true);
    let mut b = use_signal(|| true);
    rsx! {
        div {
            Link {to: Route::Home {}, "Back"}
            br{}
            form {
                onsubmit: move |_| {
                    async move {
                        info!("here");

                        let data = gloo::file::futures::read_as_bytes(&input_file()).await.unwrap();
                        let resp = remove_channel(data, r(), g(), b()).await.unwrap();
                        let blob = Blob::new_with_options(resp.as_slice(), Some("image/x-png"));

                        drop(resp);
                        let object_url = ObjectUrl::from(blob);
                        output_file.set(Some(object_url));
                        info!("here2");
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
                    accept: ".jpg, .jpeg, .png",
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
                label {
                    r#for: "r",

                    "R: "
                }
                input {
                    r#type: "checkbox",
                    name: "r",
                    checked: r(),
                    onchange: move |ev| {
                        r.set(ev.value().parse().unwrap())
                    }
                }
                br{}
                label {
                    r#for: "g",

                    "G: "
                }
                input {
                    r#type: "checkbox",
                    name: "g",
                    checked: g(), 
                    onchange: move |ev| {
                        g.set(ev.value().parse().unwrap())
                    }
                }
                br{}
                label {
                    r#for: "b",

                    "B: "
                }
                input {
                    r#type: "checkbox",
                    name: "b",
                    checked: b(),
                    onchange: move |ev| {
                        b.set(ev.value().parse().unwrap())
                    }
                }
                br{}
                button {
                    r#type: "submit",

                    "Submit"
                }
            }
            div {
                if let Some(object) = output_file() {
                    img {
                        src: object.to_string()
                    }
                }
            }
        }
    }
}

#[server]
async fn remove_channel(
    data: Vec<u8>,
    r: bool,
    g: bool,
    b: bool,
) -> Result<Vec<u8>, ServerFnError> {
    use image::DynamicImage;
    use image::GenericImageView;
    use image::{self, Pixel};
    use std::collections::HashMap;
    use std::path::Path;
    use tempfile::NamedTempFile;

    let img = image::load_from_memory(data.as_slice()).unwrap();

    let mut buffer = image::ImageBuffer::new(img.width(), img.height());

    for (x, y, pixel) in img.pixels() {
        let mut rgb_pixel = pixel.to_rgb();
        if !r {
            rgb_pixel.0[0] = 0;
        }
        if !g {
            rgb_pixel.0[1] = 0;
        }
        if !b {
            rgb_pixel.0[2] = 0;
        }
        let px = buffer.get_pixel_mut(x, y);
        *px = rgb_pixel;
    }
    let temp = NamedTempFile::new().unwrap();
    let path = temp.into_temp_path().to_str().unwrap().to_string() + ".png";

    buffer.save(path.clone()).unwrap();
    Ok(std::fs::read(path).unwrap())
}
