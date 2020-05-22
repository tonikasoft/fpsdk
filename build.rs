fn main() {
    cc::Build::new()
        .include("src/cxx")
        .file("src/cxx/fp_plugclass.cpp")
        .file("src/cxx/wrapper.cpp")
        .cpp(true)
        .flag("-std=c++11")
        .compile("fpsdk");

    println!("cargo:rerun-if-changed=src/cxx/fp_plugclass.h");
    println!("cargo:rerun-if-changed=src/cxx/wrapper.h");
    println!("cargo:rerun-if-changed=src/cxx/wrapper.cpp");
}
