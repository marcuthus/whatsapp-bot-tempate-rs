use std::{fs, io::Cursor};

use image::{
    AnimationDecoder, ImageEncoder, RgbaImage, codecs::png::PngEncoder, codecs::webp::WebPDecoder,
};
use whatsapp_rust::{
    UploadOptions,
    download::MediaType,
    media::{ImageOptions, VideoOptions, image_message, video_message},
    prelude::*,
};

use crate::{
    data::{Context, Error},
    utils::ffmpeg::{self, TempDirectory},
};

const WHITE: [u8; 3] = [255, 255, 255];
const MIN_FRAME_DELAY_MS: u32 = 20;
const DEFAULT_FRAME_DELAY_MS: u32 = 100;
const MAX_FRAMES: usize = 300;
const FRAMES_PER_SECOND: u32 = 25;

enum Media {
    Image(Vec<u8>),
    Video { mp4: Vec<u8>, seconds: u32 },
}

struct Frame {
    image: RgbaImage,
    delay_ms: u32,
}

pub async fn run(ctx: &Context) -> Result<(), Error> {
    let Some(sticker) = find_quoted_sticker(ctx) else {
        ctx.reply("Responda um sticker com !sticker2media.").await?;
        return Ok(());
    };

    let client = &ctx.message.client;
    let original = client.download(&sticker).await?;

    let media = match tokio::task::spawn_blocking(move || convert(&original)).await? {
        Ok(media) => media,
        Err(error) => {
            ctx.reply(format!(
                "Não foi possível converter este sticker. ({error})"
            ))
            .await?;
            return Ok(());
        }
    };

    let context_info = Some(Box::new(ctx.message.build_quote_context()));

    let message = match media {
        Media::Image(png) => {
            let upload = client
                .upload(png, MediaType::Image, UploadOptions::default())
                .await?;

            image_message(
                upload,
                ImageOptions {
                    mimetype: Some("image/png".to_owned()),
                    context_info,
                    ..Default::default()
                },
            )
        }
        Media::Video { mp4, seconds } => {
            let upload = client
                .upload(mp4, MediaType::Video, UploadOptions::default())
                .await?;

            video_message(
                upload,
                VideoOptions {
                    mimetype: Some("video/mp4".to_owned()),
                    duration_seconds: Some(seconds),
                    gif_playback: Some(true),
                    context_info,
                    ..Default::default()
                },
            )
        }
    };

    ctx.message.send_message(message).await?;

    Ok(())
}

fn find_quoted_sticker(ctx: &Context) -> Option<wa::message::StickerMessage> {
    ctx.message
        .message
        .get_base_message()
        .extended_text_message
        .as_option()
        .and_then(|text| text.context_info.as_option())
        .and_then(|context_info| context_info.quoted_message.as_option())
        .and_then(|quoted| quoted.get_base_message().sticker_message.as_option())
        .cloned()
}

fn convert(bytes: &[u8]) -> Result<Media, Error> {
    let mut frames = decode_frames(bytes)?;

    if frames.len() <= 1 {
        let Some(mut frame) = frames.pop() else {
            return Err("o sticker não possui imagem".into());
        };

        flatten_on_white(&mut frame.image);

        return Ok(Media::Image(encode_png(&frame.image)?));
    }

    for frame in &mut frames {
        flatten_on_white(&mut frame.image);
    }

    let total_ms = frames.iter().map(|frame| frame.delay_ms).sum::<u32>();

    Ok(Media::Video {
        mp4: frames_to_mp4(&frames)?,
        seconds: total_ms.div_ceil(1000).max(1),
    })
}

fn decode_frames(bytes: &[u8]) -> Result<Vec<Frame>, Error> {
    let decoder = WebPDecoder::new(Cursor::new(bytes))?;

    if !decoder.has_animation() {
        let image = image::load_from_memory(bytes)?.to_rgba8();

        return Ok(vec![Frame {
            image,
            delay_ms: DEFAULT_FRAME_DELAY_MS,
        }]);
    }

    decoder
        .into_frames()
        .take(MAX_FRAMES)
        .map(|frame| {
            let frame = frame?;
            let (numerator, denominator) = frame.delay().numer_denom_ms();
            let delay_ms = (f64::from(numerator) / f64::from(denominator.max(1))).round() as u32;

            Ok(Frame {
                image: frame.into_buffer(),
                delay_ms: if delay_ms == 0 {
                    DEFAULT_FRAME_DELAY_MS
                } else {
                    delay_ms.max(MIN_FRAME_DELAY_MS)
                },
            })
        })
        .collect()
}

fn flatten_on_white(image: &mut RgbaImage) {
    for pixel in image.pixels_mut() {
        let alpha = u16::from(pixel[3]);

        for (channel, background) in pixel.0.iter_mut().zip(WHITE) {
            *channel =
                ((u16::from(*channel) * alpha + u16::from(background) * (255 - alpha)) / 255) as u8;
        }

        pixel[3] = 255;
    }
}

fn encode_png(image: &RgbaImage) -> Result<Vec<u8>, Error> {
    let mut encoded = Vec::new();

    PngEncoder::new(&mut encoded).write_image(
        image.as_raw(),
        image.width(),
        image.height(),
        image::ExtendedColorType::Rgba8,
    )?;

    Ok(encoded)
}

fn concat_list(delays_ms: &[u32]) -> String {
    let mut lines = delays_ms
        .iter()
        .enumerate()
        .flat_map(|(index, delay)| {
            [
                format!("file 'frame-{index}.png'"),
                format!("duration {:.3}", f64::from(*delay) / 1000.0),
            ]
        })
        .collect::<Vec<_>>();

    if let Some(last) = delays_ms.len().checked_sub(1) {
        lines.push(format!("file 'frame-{last}.png'"));
    }

    lines.join("\n")
}

fn frames_to_mp4(frames: &[Frame]) -> Result<Vec<u8>, Error> {
    let directory = TempDirectory::create()?;

    for (index, frame) in frames.iter().enumerate() {
        fs::write(
            directory.path().join(format!("frame-{index}.png")),
            encode_png(&frame.image)?,
        )?;
    }

    let delays = frames
        .iter()
        .map(|frame| frame.delay_ms)
        .collect::<Vec<_>>();
    fs::write(directory.path().join("list.txt"), concat_list(&delays))?;

    let filter =
        format!("fps={FRAMES_PER_SECOND},scale=trunc(iw/2)*2:trunc(ih/2)*2,format=yuv420p");

    ffmpeg::run(
        directory.path(),
        &[
            "-f",
            "concat",
            "-safe",
            "0",
            "-i",
            "list.txt",
            "-vf",
            &filter,
            "-c:v",
            "libx264",
            "-preset",
            "veryfast",
            "-crf",
            "23",
            "-an",
            "-movflags",
            "+faststart",
            "out.mp4",
        ],
    )?;

    Ok(fs::read(directory.path().join("out.mp4"))?)
}

#[cfg(test)]
mod tests {
    use image::Rgba;
    use webp::{AnimEncoder, AnimFrame, WebPConfig};

    use super::*;

    fn solid(width: u32, height: u32, color: [u8; 4]) -> RgbaImage {
        RgbaImage::from_pixel(width, height, Rgba(color))
    }

    fn static_sticker() -> Vec<u8> {
        let mut image = RgbaImage::new(64, 64);

        for (x, y, pixel) in image.enumerate_pixels_mut() {
            if (16..48).contains(&x) && (16..48).contains(&y) {
                *pixel = Rgba([0, 0, 255, 255]);
            }
        }

        webp::Encoder::from_rgba(image.as_raw(), 64, 64)
            .encode_lossless()
            .to_vec()
    }

    fn animated_sticker() -> Vec<u8> {
        let colors = [[255, 0, 0, 255], [0, 255, 0, 255], [0, 0, 255, 255]];
        let images = colors.map(|color| solid(64, 64, color));

        let mut config = WebPConfig::new().unwrap();
        config.lossless = 1;

        let mut encoder = AnimEncoder::new(64, 64, &config);
        encoder.set_loop_count(0);

        for (index, image) in images.iter().enumerate() {
            encoder.add_frame(AnimFrame::from_rgba(
                image.as_raw(),
                64,
                64,
                index as i32 * 200,
            ));
        }

        encoder.encode().to_vec()
    }

    #[test]
    fn converts_a_static_sticker_to_an_opaque_png_with_a_white_background() {
        let Media::Image(png) = convert(&static_sticker()).unwrap() else {
            panic!("a static sticker must become an image");
        };

        assert_eq!(&png[1..4], b"PNG");

        let decoded = image::load_from_memory(&png).unwrap().to_rgba8();

        assert_eq!(decoded.get_pixel(2, 2).0, [255, 255, 255, 255]);
        assert_eq!(decoded.get_pixel(32, 32).0, [0, 0, 255, 255]);
    }

    #[test]
    fn decodes_every_frame_of_an_animated_sticker_with_its_delay() {
        let frames = decode_frames(&animated_sticker()).unwrap();

        assert_eq!(frames.len(), 3);
        assert!(frames.iter().all(|frame| frame.delay_ms == 200));
        let green = frames[1].image.get_pixel(0, 0).0;

        assert!(green[0] <= 2 && green[1] >= 253 && green[2] <= 2 && green[3] == 255);
    }

    #[test]
    fn builds_the_concat_list_repeating_the_last_frame() {
        assert_eq!(
            concat_list(&[120, 40]),
            "file 'frame-0.png'\nduration 0.120\nfile 'frame-1.png'\nduration 0.040\nfile 'frame-1.png'"
        );
        assert_eq!(concat_list(&[]), "");
    }

    #[test]
    fn converts_an_animated_sticker_to_an_mp4_when_ffmpeg_is_installed() {
        if !ffmpeg::is_available() {
            return;
        }

        let Media::Video { mp4, seconds } = convert(&animated_sticker()).unwrap() else {
            panic!("an animated sticker must become a video");
        };

        assert_eq!(&mp4[4..8], b"ftyp");
        assert_eq!(seconds, 1);
    }

    #[test]
    fn rejects_data_that_is_not_an_image() {
        assert!(convert(b"not an image").is_err());
    }
}
