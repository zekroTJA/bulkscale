mod utils;

use log;
use image;
use image::imageops::FilterType;
use std::{fs, process, path::Path, error::Error};
use clap::Clap;
use threadpool::ThreadPool;

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

    #[clap(long, about = "Ammount of workers in the worker pool", default_value = "5")]
    workers: usize,

    #[clap(long, about = "The filter used to scale the images", default_value = "triangle")]
    filter: String,
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

    let filter_type = match utils::filtertype_fromstr(&args.filter) {
        Ok(v) => v,
        Err(err) => return Err(err),
    };

    let input_path = Path::new(&args.input);
    if !input_path.is_dir() {
        return Err("Input path is no valid directory".into());
    }

    let dirs = match fs::read_dir(&input_path) {
        Ok(v) => v,
        Err(err) => return Err(Box::new(err)),
    };

    let in_dir = Path::new(&args.input);
    let out_dir = Path::new(&args.output);

    if !out_dir.exists() {
        match fs::create_dir_all(out_dir) {
            Ok(_) => log::info!("Output dir created"),
            Err(err) => return Err(Box::new(err)),
        }
    }

    let pool = ThreadPool::new( args.workers);

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

        let scale = args.scale;
        let in_file = in_dir.join(Path::new(&entry.file_name()));
        let output = args.output.to_owned();
        pool.execute(move || {
            let out_dir = Path::new(&output);
            match process_image(&in_file, out_dir, &scale, &filter_type) {
                Ok(_) => log::info!("Processed image {:#?}", in_file.into_os_string()),
                Err(err) => {
                    log::error!("Failed getting entry: {}", err);
                },
            }
        });
    }

    pool.join();

    Ok(())
}

fn process_image(in_file: &Path, out_dir: &Path, scale: &f32, filter_type: &FilterType) -> Result<(), Box<dyn Error>> {
    let mut img = match image::open(in_file) {
        Ok(img) => img,
        Err(err) => return Err(err.into()),
    };

    let img_buffer = img.to_rgb();

    let new_width = (img_buffer.width() as f32 * scale) as u32;
    let new_height = (img_buffer.height() as f32 * scale) as u32;

    img = img.resize(new_width, new_height, *filter_type);

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