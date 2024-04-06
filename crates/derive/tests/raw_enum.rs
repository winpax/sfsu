#![allow(dead_code)]

mod helpers;

use sfsu_derive::Hooks;

use helpers::enum_to_string;

struct DummyStruct;

#[derive(Hooks)]
enum EnumWithData {
    Test1(DummyStruct),
    Test2(DummyStruct),
}

#[test]
fn has_all_variants() {
    let variants = enum_to_string::<EnumWithDataHooks>();

    assert_eq!(variants, "test1test2");
}

#[derive(Hooks)]
enum EnumExclude {
    Test1(DummyStruct),
    #[no_hook]
    Test2(DummyStruct),
    Test3(DummyStruct),
}

#[test]
fn excludes_no_hook_variant() {
    let variants = enum_to_string::<EnumExcludeHooks>();

    assert_eq!(variants, "test1test3");
}
