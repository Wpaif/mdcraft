use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use resvg::{tiny_skia, usvg};

fn main() {
    println!("cargo:rerun-if-changed=assets/icon.svg");

    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os != "windows" {
        return;
    }

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap_or_default());
    let svg_path = manifest_dir.join("assets/icon.svg");
    if !svg_path.exists() {
        println!(
            "cargo:warning=Windows icon not embedded: file missing at {}",
            svg_path.display()
        );
        return;
    }

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap_or_default());
    let ico_path = out_dir.join("mdcraft-icon.ico");

    if let Err(err) = build_ico_from_svg(&svg_path, &ico_path) {
        println!(
            "cargo:warning=Windows icon not embedded: failed to create ico ({err})"
        );
        return;
    }

    let mut res = winresource::WindowsResource::new();
    res.set_icon(&ico_path.to_string_lossy());

    if let Err(err) = res.compile() {
        println!("cargo:warning=Windows icon resource compile failed: {err}");
    }
}

fn build_ico_from_svg(svg_path: &Path, ico_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let svg_data = fs::read(svg_path)?;

    let opts = usvg::Options::default();
    let tree = usvg::Tree::from_data(&svg_data, &opts)?;

    let icon_size: u32 = 256;
    let mut pixmap = tiny_skia::Pixmap::new(icon_size, icon_size)
        .ok_or("failed to allocate icon pixmap")?;

    let svg_size = tree.size();
    let scale_x = icon_size as f32 / svg_size.width();
    let scale_y = icon_size as f32 / svg_size.height();
    let transform = tiny_skia::Transform::from_scale(scale_x, scale_y);

    let mut pixmap_mut = pixmap.as_mut();
    resvg::render(&tree, transform, &mut pixmap_mut);

    let image = ico::IconImage::from_rgba_data(icon_size, icon_size, pixmap.data().to_vec());
    let entry = ico::IconDirEntry::encode(&image)?;

    let mut icon_dir = ico::IconDir::new(ico::ResourceType::Icon);
    icon_dir.add_entry(entry);

    let mut file = fs::File::create(ico_path)?;
    icon_dir.write(&mut file)?;

    Ok(())
}
