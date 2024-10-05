use crate::{
    color::{ColorType, ExtendedColorType},
    error::{DecodingError, ImageFormatHint, ImageResult, UnsupportedError, UnsupportedErrorKind},
    image::ImageDecoder,
    ImageError, ImageFormat, LimitSupport, Limits,
};
use jxl_oxide::{JxlImage, PixelFormat};
use std::io::Read;

/// JPEG XL decoder
pub struct JxlDecoder {
    inner: JxlImage,
    limits: Limits,
}

impl JxlDecoder {
    /// Creates a new decoder that decodes from the stream ```r```.
    pub fn new(r: impl Read) -> ImageResult<Self> {
        let inner = JxlImage::builder().read(r).map_err(|err| {
            ImageError::Decoding(DecodingError::new(ImageFormatHint::Unknown, err))
        })?;

        let limits = Limits::no_limits();

        Ok(Self { inner, limits })
    }

    fn render_frame(self, idx: usize, buf: &mut [u8]) -> ImageResult<()> {
        if let color_type @ ExtendedColorType::Cmyk8 = self.original_color_type() {
            return Err(ImageError::Unsupported(
                UnsupportedError::from_format_and_kind(
                    ImageFormat::Jxl.into(),
                    UnsupportedErrorKind::Color(color_type),
                ),
            ));
        }

        let frame = self.inner.render_frame(idx).map_err(|err| {
            ImageError::Decoding(DecodingError::new(ImageFormat::Jxl.into(), err))
        })?;

        let fb = frame.image_all_channels();

        // NOTE: channels are returned as `f32`s, so they need to be converted.
        //
        // Taken from https://github.com/tirr-c/jxl-oxide/blob/2ab55e2fd3192145e230f4623dc50225dbcc111a/crates/jxl-oxide-cli/src/output.rs#L111-L113
        for (b, s) in buf.iter_mut().zip(fb.buf()) {
            *b = (*s * 255.0 + 0.5).clamp(0.0, 255.0) as u8;
        }

        Ok(())
    }
}

impl ImageDecoder for JxlDecoder {
    fn dimensions(&self) -> (u32, u32) {
        (self.inner.width(), self.inner.height())
    }

    fn color_type(&self) -> ColorType {
        match self.inner.pixel_format() {
            PixelFormat::Gray => ColorType::L8,
            PixelFormat::Graya => ColorType::La8,
            PixelFormat::Rgb => ColorType::Rgb8,
            PixelFormat::Rgba => ColorType::Rgba8,
            PixelFormat::Cmyk => ColorType::Rgb8,
            PixelFormat::Cmyka => ColorType::Rgba8,
        }
    }

    fn original_color_type(&self) -> ExtendedColorType {
        match self.inner.pixel_format() {
            PixelFormat::Gray => ExtendedColorType::L8,
            PixelFormat::Graya => ExtendedColorType::La8,
            PixelFormat::Rgb => ExtendedColorType::Rgb8,
            PixelFormat::Rgba => ExtendedColorType::Rgba8,
            PixelFormat::Cmyk => ExtendedColorType::Cmyk8,
            PixelFormat::Cmyka => ExtendedColorType::Cmyk8,
        }
    }

    fn read_image(self, buf: &mut [u8]) -> ImageResult<()> {
        self.render_frame(0, buf)
    }

    fn read_image_boxed(self: Box<Self>, buf: &mut [u8]) -> ImageResult<()> {
        (*self).read_image(buf)
    }

    fn icc_profile(&mut self) -> ImageResult<Option<Vec<u8>>> {
        Ok(self.inner.original_icc().map(|icc| icc.to_vec()))
    }

    fn set_limits(&mut self, limits: Limits) -> ImageResult<()> {
        limits.check_support(&LimitSupport::default())?;

        let (width, height) = self.dimensions();
        limits.check_dimensions(width, height)?;

        self.limits = limits;

        Ok(())
    }
}
