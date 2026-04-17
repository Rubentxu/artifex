//! Provider adapters module.
//!
//! Contains implementations of provider traits for various AI services.

pub mod elevenlabs;
pub mod replicate_image;
pub mod replicate_video;
pub mod fal_image;
pub mod fal_video;
pub mod huggingface_image;
pub mod together_text;
pub mod kie_image;

pub use elevenlabs::ElevenLabsProvider;
pub use replicate_image::ReplicateImageProvider;
pub use replicate_video::ReplicateVideoProvider;
pub use fal_image::FalImageProvider;
pub use fal_video::FalVideoProvider;
pub use huggingface_image::HuggingFaceImageProvider;
pub use together_text::TogetherTextProvider;
pub use kie_image::KieImageProvider;