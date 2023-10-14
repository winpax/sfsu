#![allow(dead_code)]

struct DummyStruct;

#[derive(sfsu_derive::derive_hook_enum)]
enum EnumWithData {
    Test1(DummyStruct),
    Test2(DummyStruct),
}
