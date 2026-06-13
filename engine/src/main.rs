use genesis_engine::app::App;
use genesis_engine::config::WorldConfig;
use genesis_engine::rng::WorldSeed;

fn main() {
    let mut app = App::new(WorldConfig::default(), WorldSeed::default());
    app.run_startup();
    println!("Genesis Engine Initialized and Startup Completed");
}
