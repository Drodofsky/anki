// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use std::env;
use std::fmt::Write;
use std::path::PathBuf;

use anki_io::write_file_if_changed;
use anki_proto_gen::get_services;
use anki_proto_gen::CollectionService;
use anki_proto_gen::Method;
use anyhow::Context;
use anyhow::Result;
use inflections::Inflect;
use itertools::Itertools;
use prost_reflect::DescriptorPool;

/// Generates a plain, synchronous `{Name}Service` trait per protobuf
/// service (implemented directly on `Collection`), taking/returning
/// strongly-typed proto structs. No byte marshalling or dispatch
/// machinery - this crate is consumed as a plain Rust library, not
/// through a byte-in/byte-out FFI boundary.
pub fn write_rust_interface(pool: &DescriptorPool) -> Result<()> {
    let mut buf = String::new();
    buf.push_str("use crate::error::Result;");

    let (col_services, _backend_services) = get_services(pool);
    let col_services = col_services
        .into_iter()
        .filter(|s| s.name != "FrontendService")
        .collect_vec();

    for service in &col_services {
        render_collection_trait(service, &mut buf);
    }

    let buf = format_code(buf)?;
    let out_dir = env::var("OUT_DIR").unwrap();
    let path = PathBuf::from(out_dir).join("backend.rs");
    write_file_if_changed(path, buf).context("write file")?;
    Ok(())
}

fn format_code(code: String) -> Result<String> {
    let syntax_tree = syn::parse_file(&code)?;
    Ok(prettyplease::unparse(&syntax_tree))
}

fn render_collection_trait(service: &CollectionService, buf: &mut String) {
    let name = &service.name;
    writeln!(buf, "pub trait {name} {{").unwrap();
    for method in &service.trait_methods {
        render_trait_method(method, buf);
    }
    buf.push('}');
}

fn render_trait_method(method: &Method, buf: &mut String) {
    let method_name = &method.name;
    let input_with_label = method.get_input_arg_with_label();
    let output_type = method.get_output_type();
    writeln!(
        buf,
        "fn {method_name}(&mut self, {input_with_label}) -> Result<{output_type}>;"
    )
    .unwrap();
}

trait MethodHelpers {
    fn input_type(&self) -> Option<String>;
    fn output_type(&self) -> Option<String>;
    fn get_input_arg_with_label(&self) -> String;
    fn get_output_type(&self) -> String;
}

impl MethodHelpers for Method {
    fn input_type(&self) -> Option<String> {
        self.input().map(|t| rust_type(t.full_name()))
    }

    fn output_type(&self) -> Option<String> {
        self.output().map(|t| rust_type(t.full_name()))
    }

    /// No text if generic::Empty
    fn get_input_arg_with_label(&self) -> String {
        self.input_type()
            .as_ref()
            .map(|t| format!("input: {t}"))
            .unwrap_or_default()
    }

    /// () if generic::Empty
    fn get_output_type(&self) -> String {
        self.output_type().as_deref().unwrap_or("()").into()
    }
}

fn rust_type(name: &str) -> String {
    let Some((head, tail)) = name.rsplit_once('.') else {
        panic!()
    };
    format!(
        "{}::{}",
        head.to_snake_case()
            .replace('.', "::")
            .replace("anki::", "anki_proto::"),
        tail
    )
}
