use std::{fs, process, path::Path};
use clap::Clap;
use log;
use image;

macro_rules! fatal {
    ( $( $v:expr ),+ )  => {
        log::error!( $( $v ),+ );
        process::exit(1);
    }
}

#[derive(Clap)]
#[clap(version = "0.1", author = "zekro <contact@zekro.de>")]
struct Args {
    #[clap(short, long, about = "The scale of the results")]
    scale: f32,

    #[clap(short, long, about = "The directory of the input data", default_value = ".")]
    input: String,

    #[clap(short, long, about = "The directory of the output data", default_value = "output")]
    output: String,
}

fn main() {
    env_logger::init();
    
    let args = Args::parse();

    let input_path = Path::new(&args.input);
    if !input_path.is_dir() {
        fatal!("Given input path is not a directory");
    }

    let dirs = fs::read_dir(&input_path)
        .map_err(fatal_map)
        .unwrap();

    let out_dir = Path::new(&args.output);
    let in_dir = Path::new(&args.input);

    for entry in dirs {
        match entry {
            Err(e) => log::error!("Failed getting entry: {}", e),
            Ok(entry) => {
                match entry.file_type() {
                    Err(e) => log::error!("Failed getting file type: {}", e),
                    Ok(typ) => {
                        if typ.is_file() {
                            let in_file = in_dir.join(Path::new(&entry.file_name()));
                            match process_image(&in_file, &out_dir, &args.scale) {
                                Err(e) => log::error!("Failed processing image: {}", e),
                                Ok(_) => log::info!("Processed image {:#?}", in_file.into_os_string()),
                            }
                        }
                    }
                }
            },
        }
    }
}

fn process_image(in_file: &Path, out_dir: &Path, scale: &f32) -> Result<(), image::error::ImageError> {
    let mut img = match image::open(in_file) {
        Ok(img) => img,
        Err(err) => return Err(err),
    };

    let img_buffer = img.to_rgb();

    let new_width = (img_buffer.width() as f32 * scale) as u32;
    let new_height = (img_buffer.height() as f32 * scale) as u32;

    img = img.resize(new_width, new_height, image::imageops::FilterType::Gaussian);

    let file_name = in_file.file_name().unwrap(); // TODO: might panic?
    let out_file = out_dir.join(Path::new(file_name));
    match img.save(out_file) {
        Ok(_) => Ok(()),
        Err(err ) => Err(err),
    }
}

fn fatal_map(e: std::io::Error) {
    fatal!("{}", e);
}