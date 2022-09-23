mod download_fonts;
mod gen_fonts;

fn main() {
    let characters = download_fonts::download_fonts().expect("Could not download and parse fonts.");
    gen_fonts::generate_fonts(characters);
}
