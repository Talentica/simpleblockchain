extern crate prost_build;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

fn add_serialization_to_attribute_list(fullpath: &PathBuf) -> io::Result<()> {
    let line_to_be_replaced = "#[derive(Clone, PartialEq, ::prost::Message)]";
    let line_to_replace_with =
        "//// Auto-generated using build.rs and proto files. Don't edit by hand. //// \r\n#[derive(Clone, PartialEq, Serialize, Deserialize, ::prost::Message, BinaryValue, ObjectHash)]
#[binary_value(codec = \"bincode\")]";
    let mut src_file = File::open(&fullpath)?;
    let mut filedata = String::new();
    src_file.read_to_string(&mut filedata)?;
    drop(src_file);

    let new_data = filedata.replace(&line_to_be_replaced, &line_to_replace_with);
    let mut dst_file = File::create(&fullpath)?;
    dst_file.write(new_data.as_bytes())?;

    Ok(())
}

fn update_prost_generated_struct(dir: &Path) -> io::Result<()> {
    if dir.is_dir() {
        for dir_item in fs::read_dir(dir)? {
            let entry = dir_item?;
            let fullpath = entry.path();
            if fullpath.is_file() {
                match fullpath.extension().unwrap().to_str() {
                    Some("rs") => {
                        add_serialization_to_attribute_list(&fullpath)?;
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}

fn main() {
    let src = Path::new("src");
    let mut config = prost_build::Config::default();
    config.out_dir(src);
    //env::set_var("OUT_DIR", "src");
    let proto_files = ["src/proto/user_transactions.proto"];
    let includes = ["src/proto"];

    //rust file is generated in the out_dir defined during compilation of build.rs - kept in main project folder.
    //prost_build::compile_protos(&proto_files, &includes).unwrap();
    config.compile_protos(&proto_files, &includes).unwrap();

    update_prost_generated_struct(&src).unwrap();
}
