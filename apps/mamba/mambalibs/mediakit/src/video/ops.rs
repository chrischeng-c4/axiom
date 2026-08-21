//! Video frame operations — resize, convert, transform.

use super::error::{Result, VideoError};
use super::types::Frame;

/// Resize a video (all frames) using nearest-neighbor interpolation.
pub fn resize_video(
    data: &[u8],
    src_w: usize,
    src_h: usize,
    dst_w: usize,
    dst_h: usize,
) -> Result<Vec<u8>> {
    let src_frame_size = src_w * src_h * 3;
    let dst_frame_size = dst_w * dst_h * 3;

    if data.len() % src_frame_size != 0 {
        return Err(VideoError::InvalidParameter(
            "data length not aligned to frame size".into(),
        ));
    }

    let frame_count = data.len() / src_frame_size;
    let mut result = vec![0u8; dst_frame_size * frame_count];

    for f in 0..frame_count {
        let src_offset = f * src_frame_size;
        let dst_offset = f * dst_frame_size;

        for y in 0..dst_h {
            for x in 0..dst_w {
                let sx = (x * src_w) / dst_w;
                let sy = (y * src_h) / dst_h;
                let si = src_offset + (sy * src_w + sx) * 3;
                let di = dst_offset + (y * dst_w + x) * 3;
                result[di] = data[si];
                result[di + 1] = data[si + 1];
                result[di + 2] = data[si + 2];
            }
        }
    }

    Ok(result)
}

/// Convert video frames to grayscale.
pub fn to_grayscale(data: &[u8], width: usize, height: usize) -> Result<Vec<u8>> {
    let frame_size = width * height * 3;
    let gray_frame_size = width * height;

    if data.len() % frame_size != 0 {
        return Err(VideoError::InvalidParameter(
            "data length not aligned to frame size".into(),
        ));
    }

    let frame_count = data.len() / frame_size;
    let mut result = vec![0u8; gray_frame_size * frame_count];

    for f in 0..frame_count {
        let src_offset = f * frame_size;
        let dst_offset = f * gray_frame_size;

        for i in 0..width * height {
            let si = src_offset + i * 3;
            let r = data[si] as f64;
            let g = data[si + 1] as f64;
            let b = data[si + 2] as f64;
            result[dst_offset + i] = (0.299 * r + 0.587 * g + 0.114 * b) as u8;
        }
    }

    Ok(result)
}

/// Compute frame difference (motion detection).
///
/// Returns a grayscale video where each pixel is |frame[i] - frame[i-1]|.
pub fn frame_diff(data: &[u8], width: usize, height: usize) -> Result<Vec<u8>> {
    let frame_size = width * height * 3;
    if data.len() % frame_size != 0 {
        return Err(VideoError::InvalidParameter(
            "data length not aligned to frame size".into(),
        ));
    }

    let frame_count = data.len() / frame_size;
    if frame_count < 2 {
        return Err(VideoError::InvalidParameter(
            "need at least 2 frames for frame diff".into(),
        ));
    }

    let pixels = width * height;
    let mut result = vec![0u8; pixels * (frame_count - 1)];

    for f in 1..frame_count {
        let prev_offset = (f - 1) * frame_size;
        let curr_offset = f * frame_size;
        let dst_offset = (f - 1) * pixels;

        for i in 0..pixels {
            let pi = prev_offset + i * 3;
            let ci = curr_offset + i * 3;
            let dr = (data[ci] as i16 - data[pi] as i16).unsigned_abs();
            let dg = (data[ci + 1] as i16 - data[pi + 1] as i16).unsigned_abs();
            let db = (data[ci + 2] as i16 - data[pi + 2] as i16).unsigned_abs();
            result[dst_offset + i] = ((dr + dg + db) / 3).min(255) as u8;
        }
    }

    Ok(result)
}

/// Sample frames at regular intervals.
pub fn sample_frames(frames: &[Frame], count: usize) -> Vec<&Frame> {
    if frames.is_empty() || count == 0 {
        return vec![];
    }
    let step = (frames.len() as f64 / count as f64).max(1.0);
    let mut result = Vec::with_capacity(count);
    let mut pos = 0.0;
    while (pos as usize) < frames.len() && result.len() < count {
        result.push(&frames[pos as usize]);
        pos += step;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resize_video() {
        // 2x2 frame, 1 frame, scale to 4x4
        let data = vec![255, 0, 0, 0, 255, 0, 0, 0, 255, 128, 128, 128];
        let resized = resize_video(&data, 2, 2, 4, 4).unwrap();
        assert_eq!(resized.len(), 4 * 4 * 3);
    }

    #[test]
    fn test_to_grayscale() {
        let data = vec![255, 0, 0, 0, 255, 0, 0, 0, 255, 255, 255, 255];
        let gray = to_grayscale(&data, 2, 2).unwrap();
        assert_eq!(gray.len(), 4);
        assert_eq!(gray[3], 255); // white -> 255
    }

    #[test]
    fn test_frame_diff() {
        // 2 frames of 2x2, second frame is brighter
        let mut data = vec![0u8; 2 * 2 * 3 * 2];
        // Frame 2: all pixels = 100
        for i in (2 * 2 * 3)..(2 * 2 * 3 * 2) {
            data[i] = 100;
        }
        let diff = frame_diff(&data, 2, 2).unwrap();
        assert_eq!(diff.len(), 4); // 1 diff frame, grayscale
        assert_eq!(diff[0], 100); // difference = 100
    }

    #[test]
    fn test_sample_frames() {
        let frames: Vec<Frame> = (0..10)
            .map(|i| {
                let mut f = Frame::blank(2, 2);
                f.index = i;
                f
            })
            .collect();
        let sampled = sample_frames(&frames, 3);
        assert_eq!(sampled.len(), 3);
    }
}
