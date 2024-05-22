#![allow(non_snake_case)]
#![allow(unused_imports)]

use dioxus::prelude::*;
use gloo::file::{Blob, File, ObjectUrl};

use tracing::info;

use crate::Route;

#[component]
pub fn RgbToGrayscale() -> Element {
    let mut input_file = use_signal(|| File::new("input_file.jpg", Vec::<u8>::new().as_slice()));
    let mut output_file: Signal<Option<ObjectUrl>> = use_signal(|| None);

    rsx! {
        div {
            Link {to: Route::Home {}, "Back"}
            br{}
            form {
                onsubmit: move |_| {
                    async move {
                        info!("here");

                        let data = gloo::file::futures::read_as_bytes(&input_file()).await.unwrap();
                        let resp = rgb_to_grayscale(data).await.unwrap();
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

#[allow(dead_code)]
#[inline]
fn clamp<N>(a: N, min: N, max: N) -> N
where
    N: PartialOrd,
{
    if a < min {
        min
    } else if a > max {
        max
    } else {
        a
    }
}

#[server]
async fn rgb_to_grayscale(data: Vec<u8>) -> Result<Vec<u8>, ServerFnError> {
    use tempfile::NamedTempFile;
    use std::collections::HashMap;
    use std::path::Path;
    use image::GenericImageView;
    use image::{self, Pixel};
    use image::DynamicImage;

    let img = image::load_from_memory(data.as_slice()).expect("file not found!");
    let mut buffer = image::ImageBuffer::new(img.width(), img.height());

    for (x, y, pixel) in img.pixels() {
        let (r, g, b) = (pixel.0[0] as f64, pixel.0[1] as f64, pixel.0[2] as f64);
        let gray = (((0.7 * r) + (1.0 * g) + (1.0 * b)) / 3.0)
            .max(std::i64::MIN as f64)
            .min(std::i64::MAX as f64)
            .round() as i64;
        let px = buffer.get_pixel_mut(x, y);

        *px = image::Rgb([gray as u8, gray as u8, gray as u8]);
    }

    let temp = NamedTempFile::new().unwrap();
    let path = temp.into_temp_path().to_str().unwrap().to_string() + ".png";

    buffer.save(path.clone()).unwrap();
    Ok(std::fs::read(path).unwrap())
}
