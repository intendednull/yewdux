use yew_agent::Registrable;
use yewdux_idb::IndexedDbReactor;

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    IndexedDbReactor::<idb_agent::Data>::registrar().register();
}
