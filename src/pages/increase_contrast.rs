#![allow(non_snake_case)]
#![allow(unused_imports)]

use dioxus::prelude::*;
use gloo::file::{Blob, File, ObjectUrl};

use tracing::info;

use crate::Route;

#[component]
pub fn IncreaseContrast() -> Element {
    let mut input_file = use_signal(|| File::new("input_file.jpg", Vec::<u8>::new().as_slice()));
    let mut output_file: Signal<Option<ObjectUrl>> = use_signal(|| None);
    let mut ammount_to_increase = use_signal(|| 0.0 as f32);

    rsx! {
        div {
            Link {to: Route::Home {}, "Back"}
            br{}
            form {
                onsubmit: move |_| {
                    async move {
                        info!("here");

                        let data = gloo::file::futures::read_as_bytes(&input_file()).await.unwrap();
                        let resp = change_image_contrast(data, ammount_to_increase()).await.unwrap();
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
                    r#for: "ammount-to-increase",

                    "Ammount to change:"
                }
                br{}
                input {
                    required: true,
                    r#type: "number",
                    step: ".01",
                    name: "ammount-to-increase",
                    onchange: move |ev| {
                        ammount_to_increase.set(ev.value().parse().unwrap());
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

use std::collections::HashMap;


#[server]
async fn change_image_contrast(data: Vec<u8>, contrast: f32) -> Result<Vec<u8>, ServerFnError> {
    use tempfile::NamedTempFile;
    use std::path::Path;
    use calamine::{open_workbook, DataType, Reader, Xlsx};
    use image::GenericImageView;
    use image::{self, Pixel};
    use image::DynamicImage;

    let image = image::load_from_memory(data.as_slice()).unwrap();

    let (width, height) = (image.width(), image.height());
    let mut out = image::ImageBuffer::new(width, height);

    let max = 255.0;

    let percent = ((100.0 + contrast) / 100.0).powi(2);

    for (x, y, pixel) in image.pixels() {
        let f = pixel.map(|b| {
            let c: f32 = b.try_into().unwrap();

            let d = ((c / max - 0.5) * percent + 0.5) * max;
            let e = crate::helpers::clamp(d, 0.0, max);

            e as u8
        });
        out.put_pixel(x, y, f);
    }

    let temp = NamedTempFile::new().unwrap();
    let path = temp.into_temp_path().to_str().unwrap().to_string() + ".png";

    out.save(path.clone()).unwrap();
    Ok(std::fs::read(path).unwrap())
}
