//! A `mdbook` backend for generating a book in the `EPUB` format.

extern crate epub_builder;
extern crate failure;
#[macro_use]
extern crate failure_derive;
extern crate handlebars;
#[macro_use]
extern crate log;
extern crate mdbook;
extern crate mime_guess;
extern crate pulldown_cmark;
extern crate semver;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;

use failure::Error;
use mdbook::config::Config as MdConfig;
use mdbook::renderer::RenderContext;
use semver::{Version, VersionReq};
use std::fs::{create_dir_all, File};
use std::path::{Path, PathBuf};

mod config;
mod generator;
mod resources;
mod utils;

pub use crate::config::Config;
pub use crate::generator::Generator;

/// The default stylesheet used to make the rendered document pretty.
pub const DEFAULT_CSS: &str = include_str!("master.css");

/// The exact version of `mdbook` this crate is compiled against.
pub const MDBOOK_VERSION: &str = mdbook::MDBOOK_VERSION;

#[derive(Debug, Clone, PartialEq, Fail)]
#[fail(
    display = "Incompatible mdbook version, expected {} but got {}",
    expected, got
)]
struct IncompatibleMdbookVersion {
    expected: String,
    got: String,
}

/// Check that the version of `mdbook` we're called by is compatible with this
/// backend.
fn version_check(ctx: &RenderContext) -> Result<(), Error> {
    let provided_version = Version::parse(&ctx.version)?;
    let required_version = VersionReq::parse(MDBOOK_VERSION)?;

    if !required_version.matches(&provided_version) {
        let e = IncompatibleMdbookVersion {
            expected: MDBOOK_VERSION.to_string(),
            got: ctx.version.clone(),
        };

        Err(Error::from(e))
    } else {
        Ok(())
    }
}

/// Generate an `EPUB` version of the provided book.
pub fn generate(ctx: &RenderContext) -> Result<(), Error> {
    info!("Starting the EPUB generator");
    version_check(ctx)?;

    let outfile = output_filename(&ctx.destination, &ctx.config);
    trace!("Output File: {}", outfile.display());

    if !ctx.destination.exists() {
        debug!(
            "Creating destination directory ({})",
            ctx.destination.display()
        );
        create_dir_all(&ctx.destination)?;
    }

    let f = File::create(&outfile)?;
    Generator::new(ctx)?.generate(f)?;

    Ok(())
}

/// Calculate the output filename using the `mdbook` config.
pub fn output_filename(dest: &Path, config: &MdConfig) -> PathBuf {
    match config.book.title {
        Some(ref title) => dest.join(title).with_extension("epub"),
        None => dest.join("book.epub"),
    }
}
