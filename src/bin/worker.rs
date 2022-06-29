use asstosrt_wasm::worker::Worker;

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    log::debug!("worker loaded");
    Worker::init();
}
