pub struct Foo;

#[test]
fn struct_name() {
    let name = schematic::type_name!(Foo);
    assert_eq!("Foo", name.name);
    assert_eq!("type_name", name.module);
}

mod nested {
    #[test]
    fn type_in_module() {
        let name = schematic::type_name!(Foo);
        assert_eq!("Foo", name.name);
        assert_eq!("type_name::nested", name.module);
    }
}
