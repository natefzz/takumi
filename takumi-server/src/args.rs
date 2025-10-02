use clap::{Parser, command};

/// Command line arguments for the image generator server.
///
/// This struct defines the configuration options that can be passed to the server
/// when starting it up. It uses the `clap` derive macro to automatically generate
/// command line argument parsing.
#[derive(Parser, Debug, Default)]
#[command(author, version, about, long_about = None)]
pub struct Args {
  /// The port number on which the server will listen.
  ///
  /// Defaults to 3000 if not specified.
  #[arg(short, long, default_value_t = 3000)]
  pub port: u16,

  /// Glob pattern of font files to load into the server.
  #[arg(short, long)]
  pub font_glob: Option<String>,

  /// The HMAC key for integrity checking. Can be any valid UTF-8 string.
  #[cfg_attr(feature = "hmac_verify", arg(long))]
  #[cfg(feature = "hmac_verify")]
  pub hmac_key: Option<String>,
}
