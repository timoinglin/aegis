// Dev-only smoke harness: runs the autodetect functions against the live
// filesystem and prints what they find. Use to debug why a path field's
// "Detect" button is coming back empty.
//   cargo run --example autodetect_exec
use aegis_lib::services::settings_store;

fn main() {
    println!("autodetect_server_path(None) = {:?}", settings_store::autodetect_server_path(None));
    let repack = r"C:\mop_repack\MOPPREMIUM\Repack";
    println!("autodetect_server_path(Some({repack})) = {:?}", settings_store::autodetect_server_path(Some(repack)));
    println!("autodetect_repack_path(None) = {:?}", settings_store::autodetect_repack_path(None));
    let server = r"C:\mop_repack\MOPPREMIUM\Database\_Server";
    println!("autodetect_repack_path(Some({server})) = {:?}", settings_store::autodetect_repack_path(Some(server)));
    println!("autodetect_client_path() = {:?}", settings_store::autodetect_client_path());
}
