pub async fn render_image(dimensions: (i32, i32), path: &Path) -> Result<DrawTarget, String> {
    let mut target = DrawTarget::new(dimensions.0, dimensions.1);
    let mut img = match image::ImageReader::open(std::path::PathBuf::from(path)) {
        Ok(v) => match v.decode() {
            Ok(v) => v,
            Err(e) => return Err(format!("Unable to decode Image: {}", e)),
        },
        Err(e) => return Err(format!("Unable to open image path: {}", e)),
    };
    img = img.resize_to_fill(
        dimensions.0 as u32,
        dimensions.1 as u32,
        image::imageops::FilterType::Lanczos3,
    );
    img = img.thumbnail_exact(dimensions.0 as u32, dimensions.1 as u32);
    let width = img.width();
    let height = img.height();
    let rgba = img.into_rgba8();
    let pixels = rgba
        .pixels()
        .map(|v| {
            let c = v.channels();
            ((c[3] as u32) << 24) | ((c[0] as u32) << 16) | ((c[1] as u32) << 8) | (c[2] as u32)
        })
        .collect::<Vec<u32>>();
    let image = raqote::Image {
        width: width as i32,
        height: height as i32,
        data: &pixels,
    };
    target.draw_image_at(0., 0., &image, &raqote::DrawOptions::new());

    Ok(target)
}
