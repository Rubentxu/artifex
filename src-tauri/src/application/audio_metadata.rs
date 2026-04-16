//! Audio metadata extraction using symphonia.

use std::path::Path;

/// Metadata extracted from an audio file.
#[derive(Debug, Clone, Default)]
pub struct AudioMetadata {
    /// Duration in seconds (None if unknown).
    pub duration_secs: Option<f64>,
    /// Sample rate in Hz (None if unknown).
    pub sample_rate: Option<u32>,
    /// Audio format/codec name (None if unknown).
    pub format: Option<String>,
}

/// Extracts audio metadata from a file using symphonia.
///
/// Gracefully returns default values on any error (corrupt file, unsupported format, etc.)
/// This function never panics.
pub fn extract_audio_metadata(file_path: &Path) -> AudioMetadata {
    fn inner(path: &Path) -> Option<AudioMetadata> {
        // Open the file and wrap in Box for MediaSourceStream
        let file = std::fs::File::open(path).ok()?;
        let mss = symphonia::core::io::MediaSourceStream::new(Box::new(file), Default::default());

        // Probe the file format
        let meta_revisor = symphonia::default::get_probe();
        let probed = meta_revisor
            .format(
                &Default::default(),
                mss,
                &Default::default(),
                &Default::default(),
            )
            .ok()?;

        let format = probed.format;

        // Get the default audio track
        let track = format.default_track()?;

        // Get codec parameters
        let codec_params = track.codec_params.clone();

        // Extract sample rate
        let sample_rate = codec_params.sample_rate;

        // Calculate duration from n_frames / sample_rate
        let duration_secs = if let (Some(n_frames), Some(sr)) = (codec_params.n_frames, sample_rate)
        {
            if sr > 0 {
                Some(n_frames as f64 / sr as f64)
            } else {
                None
            }
        } else {
            None
        };

        // Extract format from codec id
        let format_name = format!("{:?}", codec_params.codec);

        Some(AudioMetadata {
            duration_secs,
            sample_rate,
            format: Some(format_name.to_string()),
        })
    }

    inner(file_path).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_extract_audio_metadata_nonexistent() {
        let result = extract_audio_metadata(Path::new("/nonexistent/file.mp3"));
        assert!(result.duration_secs.is_none());
        assert!(result.sample_rate.is_none());
        assert!(result.format.is_none());
    }

    #[test]
    fn test_extract_audio_metadata_corrupt_file() {
        // Write corrupt bytes that aren't a valid audio file
        let mut file = NamedTempFile::with_suffix(".mp3").unwrap();
        file.write_all(b"not a valid audio file corpus").unwrap();
        let path = file.path().to_path_buf();

        let result = extract_audio_metadata(&path);
        // Should not panic, should return defaults
        assert!(result.duration_secs.is_none());
        assert!(result.sample_rate.is_none());
        assert!(result.format.is_none());
    }
}
