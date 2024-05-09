#![allow(dead_code)]

struct DummyStruct;

impl DummyStruct {
    pub async fn run(
        self,
        _: &impl sprinkles::contexts::ScoopContext<sprinkles::config::Scoop>,
    ) -> anyhow::Result<()> {
        println!("Hello, world!");

        Ok(())
    }
}

#[derive(sfsu_derive::Runnable)]
enum MaybeIntoInner {
    Test1(DummyStruct),
    Test2(DummyStruct),
}
