fn main() {
    let html = futures_executor::block_on(relay_landing::render()).expect("Topcoat render failed");
    print!("{html}");
}
