use std::num::NonZeroU64;

fn originate(serial: NonZeroU64) -> super::Parcel {
    super::seal(super::Flavor::Alpha, serial).unwrap()
}

fn classify(frame_word: super::Parcel) -> Result<super::Outcome, super::FrameError> {
    acquire(frame_word)
}

fn expose(frame_word: super::Parcel) -> Result<super::Outcome, super::FrameError> {
    classify(frame_word)
}
