use image::{
    RgbaImage,
    imageops::{self, FilterType},
};
use whatsapp_rust::{UploadOptions, download::MediaType, prelude::*};

use crate::{
    commands::{ParseError, media::video_sticker},
    data::{Context, Error},
};

pub const STICKER_SIZE: u32 = 512;
const WEBP_QUALITY: f32 = 75.0;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum CropMode {
    #[default]
    Fill,
    Fit,
    Dented,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StickerOptions {
    pub crop: CropMode,
    pub center: bool,
}

impl Default for StickerOptions {
    fn default() -> Self {
        Self {
            crop: CropMode::default(),
            center: true,
        }
    }
}

impl StickerOptions {
    pub fn parse(crop: Option<&str>, center: Option<&str>) -> Result<Self, ParseError> {
        let crop = match crop.map(str::to_lowercase).as_deref() {
            None | Some("true") => CropMode::Fill,
            Some("false") => CropMode::Fit,
            Some("dented") => CropMode::Dented,
            Some(_) => return Err(invalid("crop", crop)),
        };

        let center = match center.map(str::to_lowercase).as_deref() {
            None | Some("true") => true,
            Some("false") => false,
            Some(_) => return Err(invalid("center", center)),
        };

        Ok(Self { crop, center })
    }
}

fn invalid(name: &'static str, value: Option<&str>) -> ParseError {
    ParseError::InvalidArgument {
        name,
        value: value.unwrap_or_default().to_owned(),
    }
}

enum Source {
    Image(wa::message::ImageMessage),
    Video(wa::message::VideoMessage),
}

pub async fn run(ctx: &Context, options: StickerOptions) -> Result<(), Error> {
    let Some(source) = find_source(ctx) else {
        ctx.reply(
            "Envie uma imagem, GIF ou vídeo com a legenda !sticker ou responda um deles com !sticker.",
        )
        .await?;
        return Ok(());
    };

    let client = &ctx.message.client;

    let (original, is_video) = match &source {
        Source::Image(image) => (client.download(image).await?, false),
        Source::Video(video) => (client.download(video).await?, true),
    };

    let converted = tokio::task::spawn_blocking(move || {
        if is_video {
            video_sticker::from_video(&original, options)
        } else {
            to_sticker(&original, options)
        }
    })
    .await?;

    let webp = match converted {
        Ok(webp) => webp,
        Err(error) if is_video => {
            ctx.reply(format!(
                "Não foi possível criar o sticker animado. ({error})"
            ))
            .await?;
            return Ok(());
        }
        Err(error) => return Err(error),
    };

    let upload = client
        .upload(webp, MediaType::Sticker, UploadOptions::default())
        .await?;

    let message = wa::Message {
        sticker_message: MessageField::some(wa::message::StickerMessage {
            url: Some(upload.url),
            direct_path: Some(upload.direct_path),
            media_key: Some(upload.media_key.to_vec()),
            file_sha256: Some(upload.file_sha256.to_vec()),
            file_enc_sha256: Some(upload.file_enc_sha256.to_vec()),
            file_length: Some(upload.file_length),
            media_key_timestamp: Some(upload.media_key_timestamp),
            mimetype: Some("image/webp".to_owned()),
            width: Some(STICKER_SIZE),
            height: Some(STICKER_SIZE),
            is_animated: Some(is_video),
            ..Default::default()
        }),
        ..Default::default()
    };

    ctx.message.send_message(message).await?;

    Ok(())
}

fn source_of(message: &wa::Message) -> Option<Source> {
    let base = message.get_base_message();

    base.image_message
        .as_option()
        .cloned()
        .map(Source::Image)
        .or_else(|| base.video_message.as_option().cloned().map(Source::Video))
}

fn find_source(ctx: &Context) -> Option<Source> {
    let message = &ctx.message.message;

    source_of(message).or_else(|| {
        message
            .get_base_message()
            .extended_text_message
            .as_option()
            .and_then(|text| text.context_info.as_option())
            .and_then(|context_info| context_info.quoted_message.as_option())
            .and_then(source_of)
    })
}

fn to_sticker(bytes: &[u8], options: StickerOptions) -> Result<Vec<u8>, Error> {
    let source = image::load_from_memory(bytes)?;

    let canvas = match options.crop {
        CropMode::Fill => fill(&source, options.center),
        CropMode::Fit => fit(&source, options.center),
        CropMode::Dented => source
            .resize_exact(STICKER_SIZE, STICKER_SIZE, FilterType::Lanczos3)
            .to_rgba8(),
    };

    let encoded =
        webp::Encoder::from_rgba(canvas.as_raw(), STICKER_SIZE, STICKER_SIZE).encode(WEBP_QUALITY);

    Ok(encoded.to_vec())
}

fn fill(source: &image::DynamicImage, center: bool) -> RgbaImage {
    let scale = f64::max(
        f64::from(STICKER_SIZE) / f64::from(source.width()),
        f64::from(STICKER_SIZE) / f64::from(source.height()),
    );
    let width = ((f64::from(source.width()) * scale).round() as u32).max(STICKER_SIZE);
    let height = ((f64::from(source.height()) * scale).round() as u32).max(STICKER_SIZE);

    let scaled = source
        .resize_exact(width, height, FilterType::Lanczos3)
        .to_rgba8();

    let (x, y) = offset(center, width - STICKER_SIZE, height - STICKER_SIZE);

    imageops::crop_imm(&scaled, x, y, STICKER_SIZE, STICKER_SIZE).to_image()
}

fn fit(source: &image::DynamicImage, center: bool) -> RgbaImage {
    let scaled = source
        .resize(STICKER_SIZE, STICKER_SIZE, FilterType::Lanczos3)
        .to_rgba8();

    let mut canvas = RgbaImage::new(STICKER_SIZE, STICKER_SIZE);
    let (x, y) = offset(
        center,
        STICKER_SIZE - scaled.width(),
        STICKER_SIZE - scaled.height(),
    );
    imageops::overlay(&mut canvas, &scaled, i64::from(x), i64::from(y));

    canvas
}

fn offset(center: bool, horizontal_space: u32, vertical_space: u32) -> (u32, u32) {
    if center {
        (horizontal_space / 2, vertical_space / 2)
    } else {
        (0, 0)
    }
}

#[cfg(test)]
mod tests {
    use image::{ImageFormat, Rgba};

    use super::*;

    fn half_red_half_blue(width: u32, height: u32) -> Vec<u8> {
        let mut image = RgbaImage::new(width, height);

        for (x, _, pixel) in image.enumerate_pixels_mut() {
            *pixel = if x < width / 2 {
                Rgba([255, 0, 0, 255])
            } else {
                Rgba([0, 0, 255, 255])
            };
        }

        let mut bytes = std::io::Cursor::new(Vec::new());
        image.write_to(&mut bytes, ImageFormat::Png).unwrap();
        bytes.into_inner()
    }

    fn sticker(options: StickerOptions) -> RgbaImage {
        let webp = to_sticker(&half_red_half_blue(400, 200), options).unwrap();

        image::load_from_memory(&webp).unwrap().to_rgba8()
    }

    fn is_red(pixel: &Rgba<u8>) -> bool {
        pixel[0] > 128 && pixel[2] < 128
    }

    fn is_blue(pixel: &Rgba<u8>) -> bool {
        pixel[2] > 128 && pixel[0] < 128
    }

    fn options(crop: CropMode, center: bool) -> StickerOptions {
        StickerOptions { crop, center }
    }

    #[test]
    fn defaults_to_a_centered_fill() {
        assert_eq!(StickerOptions::default(), options(CropMode::Fill, true));
        assert_eq!(
            StickerOptions::parse(None, None),
            Ok(StickerOptions::default())
        );
    }

    #[test]
    fn parses_crop_and_center_values_ignoring_case() {
        assert_eq!(
            StickerOptions::parse(Some("FALSE"), Some("False")),
            Ok(options(CropMode::Fit, false))
        );
        assert_eq!(
            StickerOptions::parse(Some("dented"), None),
            Ok(options(CropMode::Dented, true))
        );
        assert_eq!(
            StickerOptions::parse(Some("true"), Some("true")),
            Ok(StickerOptions::default())
        );
    }

    #[test]
    fn rejects_unknown_values() {
        assert!(matches!(
            StickerOptions::parse(Some("maybe"), None),
            Err(ParseError::InvalidArgument { name: "crop", .. })
        ));
        assert!(matches!(
            StickerOptions::parse(None, Some("dented")),
            Err(ParseError::InvalidArgument { name: "center", .. })
        ));
    }

    #[test]
    fn fill_centered_keeps_both_halves() {
        let result = sticker(options(CropMode::Fill, true));

        assert_eq!(result.dimensions(), (512, 512));
        assert!(is_red(result.get_pixel(10, 256)));
        assert!(is_blue(result.get_pixel(500, 256)));
    }

    #[test]
    fn fill_anchored_keeps_only_the_left_side() {
        let result = sticker(options(CropMode::Fill, false));

        assert!(is_red(result.get_pixel(10, 256)));
        assert!(is_red(result.get_pixel(500, 256)));
    }

    #[test]
    fn fit_adds_transparent_bars_around_the_image() {
        let centered = sticker(options(CropMode::Fit, true));
        let anchored = sticker(options(CropMode::Fit, false));

        assert_eq!(centered.get_pixel(256, 5)[3], 0);
        assert!(centered.get_pixel(256, 256)[3] > 0);
        assert!(anchored.get_pixel(256, 5)[3] > 0);
        assert_eq!(anchored.get_pixel(256, 500)[3], 0);
    }

    #[test]
    fn dented_stretches_the_whole_image() {
        let result = sticker(options(CropMode::Dented, true));

        assert!(is_red(result.get_pixel(10, 5)));
        assert!(is_blue(result.get_pixel(500, 506)));
    }

    #[test]
    fn rejects_data_that_is_not_an_image() {
        assert!(to_sticker(b"not an image", StickerOptions::default()).is_err());
    }
}
