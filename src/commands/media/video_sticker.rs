use std::fs;

use crate::{
    commands::media::sticker::{CropMode, STICKER_SIZE, StickerOptions},
    data::Error,
    utils::ffmpeg::{self, TempDirectory},
};

const MAX_DURATION_SECONDS: &str = "10";
const MAX_STICKER_BYTES: usize = 500 * 1024;

const ATTEMPTS: [(u32, u32); 4] = [(15, 60), (12, 45), (10, 30), (8, 20)];

pub fn build_filter(options: StickerOptions, fps: u32) -> String {
    let size = STICKER_SIZE;

    match options.crop {
        CropMode::Dented => format!("fps={fps},scale={size}:{size}"),
        CropMode::Fit => {
            let offset = if options.center {
                "(ow-iw)/2:(oh-ih)/2"
            } else {
                "0:0"
            };

            format!(
                "fps={fps},scale={size}:{size}:force_original_aspect_ratio=decrease,format=rgba,pad={size}:{size}:{offset}:color=black@0"
            )
        }
        CropMode::Fill => {
            let origin = if options.center { "" } else { ":0:0" };

            format!(
                "fps={fps},scale={size}:{size}:force_original_aspect_ratio=increase,crop={size}:{size}{origin}"
            )
        }
    }
}

pub fn from_video(bytes: &[u8], options: StickerOptions) -> Result<Vec<u8>, Error> {
    let directory = TempDirectory::create()?;
    fs::write(directory.path().join("input"), bytes)?;

    let mut last_size = 0;

    for (fps, quality) in ATTEMPTS {
        let filter = build_filter(options, fps);
        let quality = quality.to_string();

        ffmpeg::run(
            directory.path(),
            &[
                "-t",
                MAX_DURATION_SECONDS,
                "-i",
                "input",
                "-vf",
                &filter,
                "-c:v",
                "libwebp",
                "-q:v",
                &quality,
                "-loop",
                "0",
                "-an",
                "out.webp",
            ],
        )?;

        let output = fs::read(directory.path().join("out.webp"))?;

        if output.len() <= MAX_STICKER_BYTES {
            return Ok(output);
        }

        last_size = output.len();
    }

    Err(
        format!("o sticker animado ficou com {last_size} bytes (limite: {MAX_STICKER_BYTES})")
            .into(),
    )
}

#[cfg(test)]
mod tests {
    use image::{AnimationDecoder, codecs::webp::WebPDecoder};

    use super::*;

    fn options(crop: CropMode, center: bool) -> StickerOptions {
        StickerOptions { crop, center }
    }

    fn test_video() -> Option<Vec<u8>> {
        if !ffmpeg::is_available() {
            return None;
        }

        let directory = TempDirectory::create().unwrap();

        ffmpeg::run(
            directory.path(),
            &[
                "-f",
                "lavfi",
                "-i",
                "testsrc=duration=2:size=320x180:rate=20",
                "-pix_fmt",
                "yuv420p",
                "video.mp4",
            ],
        )
        .unwrap();

        Some(fs::read(directory.path().join("video.mp4")).unwrap())
    }

    #[test]
    fn fills_the_square_and_crops_centered_by_default() {
        assert_eq!(
            build_filter(StickerOptions::default(), 15),
            "fps=15,scale=512:512:force_original_aspect_ratio=increase,crop=512:512"
        );
    }

    #[test]
    fn anchors_the_crop_to_the_top_left_when_not_centered() {
        assert!(build_filter(options(CropMode::Fill, false), 12).ends_with("crop=512:512:0:0"));
    }

    #[test]
    fn fits_the_whole_video_with_transparent_bars() {
        let centered = build_filter(options(CropMode::Fit, true), 10);
        let anchored = build_filter(options(CropMode::Fit, false), 10);

        assert!(centered.contains("pad=512:512:(ow-iw)/2:(oh-ih)/2:color=black@0"));
        assert!(anchored.contains("pad=512:512:0:0:color=black@0"));
    }

    #[test]
    fn stretches_the_video_when_dented() {
        assert_eq!(
            build_filter(options(CropMode::Dented, true), 8),
            "fps=8,scale=512:512"
        );
    }

    #[test]
    fn creates_an_animated_webp_that_fits_the_limit_when_ffmpeg_is_installed() {
        let Some(video) = test_video() else {
            return;
        };

        for crop in [CropMode::Fill, CropMode::Fit, CropMode::Dented] {
            let webp = from_video(&video, options(crop, true)).unwrap();
            let decoder = WebPDecoder::new(std::io::Cursor::new(webp.as_slice())).unwrap();

            assert!(webp.len() <= MAX_STICKER_BYTES);
            assert!(decoder.has_animation());
            assert_eq!(decoder.into_frames().count(), 30);
        }
    }

    #[test]
    fn rejects_data_that_is_not_a_video_when_ffmpeg_is_installed() {
        if ffmpeg::is_available() {
            assert!(from_video(b"not a video", StickerOptions::default()).is_err());
        }
    }
}
