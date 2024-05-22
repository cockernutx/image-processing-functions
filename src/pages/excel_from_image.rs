#![allow(non_snake_case)]

use dioxus::prelude::*;
use gloo::file::{Blob, File, ObjectUrl};

use tracing::info;

use crate::Route;

#[component]
pub fn ExcelFromImage() -> Element {
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
                        let resp = image_to_excel(data).await.unwrap();
                        let blob = Blob::new_with_options(resp.as_slice(), Some("application/vnd.ms-excel"));
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
                    a {href: object.to_string(), "Download excel"}
                }
            }
        }
    }
}

#[server]
async fn image_to_excel(data: Vec<u8>) -> Result<Vec<u8>, ServerFnError> {
    use std::collections::HashMap;
    use std::path::Path;
    use tempfile::NamedTempFile;

    use calamine::{open_workbook, DataType, Reader, Xlsx};
    use image::GenericImageView;
    use image::{self, Pixel};
    use crate::rgb::*;
    use rust_xlsxwriter::*;

    const WORKSHEET_NAME: &str = "Image matrix";

    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    worksheet
        .set_name(WORKSHEET_NAME)
        .expect("Could not set the worksheet name");

    let img = image::load_from_memory(data.as_slice()).expect("file not found!");
    for (x, y, pixel) in img.pixels() {
        let hex = format!(
            "#{}",
            rgb(pixel[0].into(), pixel[1].into(), pixel[2].into())
        );
        let cell_format = Format::new().set_background_color(hex.as_str());

        worksheet
            .write_with_format(
                y,
                x.try_into().unwrap(),
                format!("R: {}; G: {}; B: {};", pixel[0], pixel[1], pixel[2]),
                &cell_format,
            )
            .unwrap();
    }

    let temp = NamedTempFile::new().unwrap();
    let path = temp.into_temp_path().to_str().unwrap().to_string() + ".xlsx";
    workbook.save(&path).unwrap();
    Ok(std::fs::read(path).unwrap())
}
