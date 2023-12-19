fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<idb_agent::App>::new().render();
}
