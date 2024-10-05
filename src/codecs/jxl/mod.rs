//! Decoding and Encoding of JPEG XL Images
//!
//! JPEG XL (Joint Photographic Experts Group, XL) is a royalty-free raster-graphics
//! file format that supports both lossy and lossless compression.
//! This module implements the JPEG XL standard.
//!
//! # Related Links
//! * <https://ds.jpeg.org/whitepapers/jpeg-xl-whitepaper.pdf> - The JPEG XL whitepaper
//!

pub use self::decoder::JxlDecoder;

mod decoder;
