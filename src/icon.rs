use eframe::egui::{Context, TextureHandle, ColorImage, TextureOptions};
use image::GenericImageView;

pub fn load_icon_texture(ctx: &Context) -> TextureHandle {
    let image = image::open("assets/icon256.png").expect("Failed to load icon");
    let image = image.resize(128, 128, image::imageops::FilterType::Lanczos3);
    let rgba = image.to_rgba8();
    let size = [128, 128];
    let pixels = rgba.into_vec();
    ctx.load_texture(
        "about_icon",
        ColorImage::from_rgba_unmultiplied(size, &pixels),
        TextureOptions::LINEAR,
    )
}

