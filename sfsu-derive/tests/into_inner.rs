#![allow(dead_code)]

struct DummyStruct;

impl DummyStruct {
    pub fn run(self) -> anyhow::Result<()> {
        println!("Hello, world!");

        Ok(())
    }
}

#[derive(sfsu_derive::Runnable)]
enum MaybeIntoInner {
    Test1(DummyStruct),
    Test2(DummyStruct),
}
