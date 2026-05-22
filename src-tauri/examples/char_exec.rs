// Dev-only smoke harness: exercises characters::list (no AppHandle needed).
use aegis_lib::models::Settings;
use aegis_lib::services::characters;

fn main() {
    let mut s = Settings::default();
    s.server_path = Some(r"C:\mop_repack\MOPPREMIUM\Database\_Server".into());
    s.repack_path = Some(r"C:\mop_repack\MOPPREMIUM\Repack".into());
    let chars = characters::list(&s);
    println!("count={}", chars.len());
    for c in chars.iter().take(3) {
        println!("{c:?}");
    }
}
