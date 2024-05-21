#![allow(non_snake_case)]
#![allow(unused_imports)]

use dioxus::prelude::*;
use gloo::file::{Blob, File, ObjectUrl};

use tracing::info;

use crate::Route;

#[component]
pub fn ReplaceColor() -> Element {
    let mut input_file = use_signal(|| File::new("input_file.jpg", Vec::<u8>::new().as_slice()));
    let mut output_file: Signal<Option<ObjectUrl>> = use_signal(|| None);
    let mut old_rgb = use_signal(|| (0 as u8, 0 as u8, 0 as u8));
    let mut new_rgb = use_signal(|| (0 as u8, 0 as u8, 0 as u8));

    rsx! {
        div {
            Link {to: Route::Home {}, "Back"}
            br{}
            form {
                onsubmit: move |_| {
                    async move {
                        info!("here");

                        let data = gloo::file::futures::read_as_bytes(&input_file()).await.unwrap();
                        let resp = replace_pixel(data, old_rgb(), new_rgb()).await.unwrap();
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
                    r#for: "old-rgb",

                    "Old RGB: "
                }
                br{}
                input {
                    r#type: "color",
                    name: "old-rgb",
                    onchange: move |ev| {
                        let v = crate::rgb::hex_to_rgb(ev.value()).unwrap();
                        old_rgb.set(v);
                    }
                }

                br{}
                br{}
                label {
                    r#for: "new-rgb",

                    "New RGB: "
                }
                br{}
                input {
                    r#type: "color",
                    name: "new-rgb",
                    onchange: move |ev| {
                        let v = crate::rgb::hex_to_rgb(ev.value()).unwrap();
                        new_rgb.set(v);
                    }
                }

                br{}br{}
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
async fn replace_pixel(
    data: Vec<u8>,
    old_color: (u8, u8, u8),
    new_color: (u8, u8, u8),
) -> Result<Vec<u8>, ServerFnError> {
    use tempfile::NamedTempFile;
    use std::collections::HashMap;
    use std::path::Path;
    use image::GenericImageView;
    use image::{self, Pixel};
    use image::DynamicImage;

    let img = image::load_from_memory(data.as_slice()).unwrap();

    let mut buffer = image::ImageBuffer::new(img.width(), img.height());

    for (x, y, pixel) in img.pixels() {
        let mut rgb_pixel = pixel.to_rgb();
        if rgb_pixel.0[0] == old_color.0
            && rgb_pixel.0[1] == old_color.1
            && rgb_pixel.0[2] == old_color.2
        {
            rgb_pixel = image::Rgb([new_color.0, new_color.1, new_color.2]);
        }
        let px = buffer.get_pixel_mut(x, y);
        *px = rgb_pixel;
    }
    let temp = NamedTempFile::new().unwrap();
    let path = temp.into_temp_path().to_str().unwrap().to_string() + ".png";

    buffer.save(path.clone()).unwrap();
    Ok(std::fs::read(path).unwrap())
}
