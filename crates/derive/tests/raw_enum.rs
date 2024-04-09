#![allow(dead_code)]

use sfsu_derive::Hooks;

use strum::IntoEnumIterator;

struct DummyStruct;

#[derive(Hooks)]
enum EnumWithData {
    Test1(DummyStruct),
    Test2(DummyStruct),
}

#[test]
fn has_all_variants() {
    let variants = EnumWithDataHooks::iter()
        .map(|v| v.hook())
        .collect::<String>();

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
    let variants = EnumExcludeHooks::iter()
        .map(|v| v.hook())
        .collect::<String>();

    assert_eq!(variants, "test1test3");
}
