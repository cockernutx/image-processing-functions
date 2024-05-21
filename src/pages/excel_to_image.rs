#![allow(non_snake_case)]

use dioxus::prelude::*;
use gloo::file::{Blob, File, ObjectUrl};

use tracing::info;

use crate::Route;

#[component]
pub fn ExcelToImage() -> Element {
    let mut input_file = use_signal(|| File::new("input_file.xls", Vec::<u8>::new().as_slice()));
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
                        let resp = excel_to_image(data).await.unwrap();
                        let blob = Blob::new_with_options(resp.as_slice(), Some("image/x-png"));
                        drop(resp);
                        let object_url = ObjectUrl::from(blob);
                        output_file.set(Some(object_url));
                        info!("here2");
                    }
                },
                label {
                    r#for: "input-file",

                    "Select an file"
                }
                br{}
                input {
                    required: true,
                    r#type: "file",
                    accept: ".xls, .xlsx",
                    name: "input-file",
                    onchange: move |ev| {
                        async move {
                            if let Some(fe) = ev.files() {
                                let files = fe.files();
                                let f = files.first().unwrap();

                                if let Some(f) = fe.read_file(f).await
                                {
                                    input_file.set(File::new("input_file.xls", f.as_slice()));
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

#[server]
async fn excel_to_image(data: Vec<u8>) -> Result<Vec<u8>, ServerFnError> {
    use std::collections::HashMap;
    use std::path::Path;
    use tempfile::NamedTempFile;

    use crate::rgb::*;
    use calamine::{open_workbook, DataType, Reader, Xlsx};
    use image::GenericImageView;
    use image::{self, Pixel};
    use rust_xlsxwriter::*;
    use std::io::{self, Write, Read};

    const WORKSHEET_NAME: &str = "Image matrix"; 

    let mut temp = NamedTempFile::new().unwrap();
    temp.write_all(data.as_slice());

    let mut workbook: Xlsx<_> =
    open_workbook(temp.path()).expect("Could not open the excel workbook");
    let r = workbook.worksheet_range(WORKSHEET_NAME).unwrap();

    let x = r.height();
    let y = r.width();

    let mut img_buffer = image::ImageBuffer::new(y.try_into().unwrap(), x.try_into().unwrap());
    for (y, x, info) in r.cells() {
        let px = img_buffer.get_pixel_mut(x.try_into().unwrap(), y.try_into().unwrap());
        let cell_text = info.get_string().unwrap();
        let mut rgb = cell_text.split(";");
        let r: u32 = rgb
            .next()
            .unwrap()
            .replace("R:", "")
            .trim()
            .parse()
            .unwrap();
        let g: u32 = rgb
            .next()
            .unwrap()
            .replace("G:", "")
            .trim()
            .parse()
            .unwrap();
        let b: u32 = rgb
            .next()
            .unwrap()
            .replace("B:", "")
            .trim()
            .parse()
            .unwrap();
        *px = image::Rgb([r as u8, g as u8, b as u8])
    }
    let temp = NamedTempFile::new().unwrap();
    let path = temp.into_temp_path().to_str().unwrap().to_string() + ".png";

    img_buffer.save(path.clone()).unwrap();
    Ok(std::fs::read(path).unwrap())
}
