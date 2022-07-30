use simplelog::{WriteLogger, LevelFilter};

use usdpl_back::Instance;
use usdpl_back::core::serdes::Primitive;

const PORT: u16 = 54321; // TODO replace with something unique

const PACKAGE_NAME: &'static str = env!("CARGO_PKG_NAME");
const PACKAGE_VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn main() -> Result<(), ()> {
    let log_filepath = format!("/tmp/{}.log", PACKAGE_NAME);
    WriteLogger::init(
        #[cfg(debug_assertions)]{LevelFilter::Debug},
        #[cfg(not(debug_assertions))]{LevelFilter::Info},
        Default::default(),
        std::fs::File::create(&log_filepath).unwrap()
    ).unwrap();

    log::info!("Starting back-end ({} v{})", PACKAGE_NAME, PACKAGE_VERSION);
    println!("Starting back-end ({} v{})", PACKAGE_NAME, PACKAGE_VERSION);
    Instance::new(PORT)
        .register("hello", |_: Vec<Primitive>| vec![format!("Hello {}", PACKAGE_NAME).into()])
        .run_blocking()
}
