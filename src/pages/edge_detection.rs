#![allow(non_snake_case)]
#![allow(unused_imports)]

use dioxus::prelude::*;
use gloo::file::{Blob, File, ObjectUrl};

use tracing::info;

use crate::Route;

#[component]
pub fn EdgeDetection() -> Element {
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
                        let resp = detect(data, ammount_to_increase() ).await.unwrap();
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

                    "Sigma:"
                }
                br{}
                input {
                    required: true,
                    r#type: "number",
                    step: ".1",
                    name: "ammount-to-increase",
                    onchange: move |ev| {
                        ammount_to_increase.set(ev.value().parse().unwrap() );
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
async fn detect(data: Vec<u8>, sigma: f32) -> Result<Vec<u8>, ServerFnError> {
    use calamine::{open_workbook, DataType, Reader, Xlsx};
    use image::{self, Pixel};
    use image::{DynamicImage, GenericImageView};
    use rust_xlsxwriter::*;
    use std::collections::HashMap;
    use std::path::Path;
    use tempfile::NamedTempFile;
    use crate::helpers::*;
    
    
    let img = image::load_from_memory(data.as_slice()).unwrap();

    let out = filter3x3(&img, &[-1.0, -1.0, -1.0, -1.0, 8.0, -1.0, -1.0, -1.0, -1.0], Some(sigma));  
    let temp = NamedTempFile::new().unwrap();
    let path = temp.into_temp_path().to_str().unwrap().to_string() + ".png";

    out.save(path.clone()).unwrap();
    Ok(std::fs::read(path).unwrap())
}
