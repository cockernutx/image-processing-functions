#![allow(non_snake_case)]
#![allow(unused_imports)]

use dioxus::prelude::*;
use gloo::file::{Blob, File, ObjectUrl};
#[cfg(features = "server")]
use image::DynamicImage;
use tracing::info;

use crate::Route;

#[component]
pub fn RgbToCmyk() -> Element {
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
                        let resp = rgb_to_cmyk(data).await.unwrap();
                        let blob = Blob::new_with_options(resp.as_slice(), Some("image/tiff"));

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

                    a {href: object.to_string(), "Download tiff (cmyk) image"}
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

use std::collections::HashMap;
#[allow(dead_code)]
type Cmyk = HashMap<(u32, u32), (f64, f64, f64, f64)>;

#[server]
async fn rgb_to_cmyk(data: Vec<u8>) -> Result<Vec<u8>, ServerFnError> {
    use tempfile::NamedTempFile;

    use image::DynamicImage;
    use image::GenericImageView; 
    use image::{self, Pixel};
    use std::path::Path;

    let img = image::load_from_memory(data.as_slice()).expect("file not found!");

    let mut cmyk = Cmyk::new();

    for (pixel_x, pixel_y, pixel) in img.pixels() {
        let rgb_pixel = pixel.to_rgb();
        let (r, g, b) = (rgb_pixel.0[0], rgb_pixel.0[1], rgb_pixel.0[2]);

        let mut k = 0.0;

        if r == 0 && g == 0 && b == 0 {
            cmyk.insert((pixel_x, pixel_y), (0.0, 0.0, 0.0, 1.0));
            continue;
        }

        let mut c = 1.0 - (r as f64 / 255.0);
        let mut m = 1.0 - (g as f64 / 255.0);
        let mut y = 1.0 - (b as f64 / 255.0);

        let min_cmy = c.min(m.min(y));

        c = (c - min_cmy) / (1.0 - min_cmy);
        m = (m - min_cmy) / (1.0 - min_cmy);
        y = (y - min_cmy) / (1.0 - min_cmy);
        k = min_cmy;

        cmyk.insert((pixel_x, pixel_y), (c, m, y, k));
    }
    let temp = NamedTempFile::new().unwrap();
    let path = temp.into_temp_path().to_str().unwrap().to_string() + ".tiff";

    img.save(path.clone()).unwrap();
    Ok(std::fs::read(path).unwrap())
}
