use std::{fs, process, path::Path, error::Error};
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

    #[clap(long, about = "Log level", default_value = "info")]
    loglevel: log::LevelFilter,
}

fn main() {
    match try_main() {
        Ok(_) => {},
        Err(err) => {
            log::error!("FATAL: {}", err);
            process::exit(1);
        }
    }
}

fn try_main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    env_logger::Builder::new()
        .filter_level(args.loglevel)
        .init();

    let input_path = Path::new(&args.input);
    if !input_path.is_dir() {
        fatal!("Given input path is not a directory");
    }

    let dirs = match fs::read_dir(&input_path) {
        Ok(v) => v,
        Err(err) => return Err(Box::new(err)),
    };

    let out_dir = Path::new(&args.output);
    let in_dir = Path::new(&args.input);

    for entry in dirs {
        let entry = match entry {
            Ok(v) => v,
            Err(err) => {
                log::error!("Failed getting entry: {}", err);
                continue;
            },
        };

        let file_type = match entry.file_type() {
            Ok(v) => v,
            Err(err) => {
                log::error!("Failed getting file type: {}", err);
                continue;
            },
        };

        if !file_type.is_file() {
            continue;
        }

        let in_file = in_dir.join(Path::new(&entry.file_name()));
        match process_image(&in_file, &out_dir, &args.scale) {
            Ok(_) => log::info!("Processed image {:#?}", in_file.into_os_string()),
            Err(err) => {
                log::error!("Failed getting entry: {}", err);
                continue;
            },
        }
    }

    Ok(())
}

fn process_image(in_file: &Path, out_dir: &Path, scale: &f32) -> Result<(), Box<dyn Error>> {
    let mut img = match image::open(in_file) {
        Ok(img) => img,
        Err(err) => return Err(err.into()),
    };

    let img_buffer = img.to_rgb();

    let new_width = (img_buffer.width() as f32 * scale) as u32;
    let new_height = (img_buffer.height() as f32 * scale) as u32;

    img = img.resize(new_width, new_height, image::imageops::FilterType::Gaussian);

    let file_name = match in_file.file_name() {
        Some(v) => v,
        None => return Err("Failed capturing file name".into()),
    };

    let out_file = out_dir.join(Path::new(file_name));
    match img.save(out_file) {
        Ok(_) => Ok(()),
        Err(err ) => Err(err.into()),
    }
}