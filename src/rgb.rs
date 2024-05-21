#![allow(dead_code)]

pub fn to_hex(value: i32) -> String {
    let value = if value > 255 {
        255
    } else if value < 0 {
        0
    } else {
        value
    };
    format!("{:02X}", value)
}

pub fn rgb(r: i32, g: i32, b: i32) -> String {
    format!("{}{}{}", to_hex(r), to_hex(g), to_hex(b))
}

pub fn hex_to_rgb(hex: String) -> Option<(u8, u8, u8)> {
    let regex = regex::Regex::new("^#?([a-f\\d]{2})([a-f\\d]{2})([a-f\\d]{2})$").unwrap();
    let res = regex.captures(&hex).unwrap();

    let r_string = res.get(1).unwrap().as_str();
    let g_string = res.get(2).unwrap().as_str();
    let b_string = res.get(3).unwrap().as_str();

    Some((
        r_string
            .parse()
            .unwrap_or(u8::from_str_radix(r_string, 16).unwrap()),
        g_string
            .parse()
            .unwrap_or(u8::from_str_radix(g_string, 16).unwrap()),
        b_string
            .parse()
            .unwrap_or(u8::from_str_radix(b_string, 16).unwrap()),
    ))
}
