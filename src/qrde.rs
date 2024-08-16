use image;

fn main() {
    let img = image::open("/home/viscanum853/QRscanner/test/data/solpgqr.png")
        .unwrap()
        .to_luma8();
    let mut img = rqrr::PreparedImage::prepare(img);
    let grids = img.detect_grids();
    let (meta, content) = grids[0].decode().unwrap();
    if !content.is_empty() {
        webbrowser::open(&content).unwrap();
    }
}
