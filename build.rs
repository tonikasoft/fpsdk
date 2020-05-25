fn main() {
    cc::Build::new()
        .include("src/cxx")
        .file("src/cxx/fp_plugclass.cpp")
        .file("src/cxx/wrapper.cpp")
        .file("src/cxx/add_child_window.mm")
        .cpp(true)
        // .flag("-std=c++11")
        // .flag("-fobjc-arc").flag("-framework").flag("Foundation")
        .compile("fpsdk");

    println!("cargo:rerun-if-changed=src/cxx/fp_plugclass.h");
    println!("cargo:rerun-if-changed=src/cxx/wrapper.h");
    println!("cargo:rerun-if-changed=src/cxx/wrapper.cpp");
}
