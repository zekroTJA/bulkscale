use std::error::Error;
use image::imageops::FilterType;

pub fn filtertype_fromstr(val: &String) -> Result<FilterType, Box<dyn Error>> {
    let val = val.to_lowercase();

    match val.as_str() {
        "nearest" => Ok(FilterType::Nearest),
        "triangle" => Ok(FilterType::Triangle),
        "catmullrom" => Ok(FilterType::CatmullRom),
        "gaussian" => Ok(FilterType::Gaussian),
        "lanczos3" => Ok(FilterType::Lanczos3),
        _ => Err("invalid filter type".into()),
    }
}