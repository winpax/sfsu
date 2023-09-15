#![allow(dead_code)]

struct DummyStruct;

#[derive(sfsu_derive::RawEnum)]
enum EnumWithData {
    Test1(DummyStruct),
    Test2(DummyStruct),
}
