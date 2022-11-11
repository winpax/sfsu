use sfst::config::ScoopConfig;

fn main() {
    let mut config = ScoopConfig::read().unwrap();
    config.update_last_update_time();
    config.save().unwrap();
}
