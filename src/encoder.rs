use crate::picture::Picture;
use anyhow::Result;
use gif::Frame;

pub fn encode_gif(path: &str, pictures: &[Picture]) -> Result<()> {
    let file = std::fs::File::create(path)?;

    let mut encoder = gif::Encoder::new(
        file,
        pictures.first().unwrap().width() as u16,
        pictures.first().unwrap().height() as u16,
        &[],
    )?;

    encoder.set_repeat(gif::Repeat::Infinite)?;

    let width = pictures.first().unwrap().width() as u16;
    let height = pictures.first().unwrap().height() as u16;

    for pic in pictures {
        let mut data = pic.data().to_owned();
        let mut frame = Frame::from_rgba_speed(width, height, data.as_mut(), 10);
        frame.delay = 6;
        encoder.write_frame(&frame)?;
    }

    Ok(())
}
