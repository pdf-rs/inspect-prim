extern crate gfx;
extern crate gfx_window_glutin;
extern crate glutin;
#[macro_use]
extern crate imgui;
extern crate imgui_gfx_renderer;

extern crate pdf;

pub mod search;
use search::*;

use imgui::*;
use std::str;

use pdf::*;
use object::*;
use backend::*;
use primitive::*;

use std::cell::RefCell;

mod support_gfx;

const CLEAR_COLOR: [f32; 4] = [0.2, 0.2, 0.3, 1.0];

fn main() {
    let backend = Vec::<u8>::open("files/minimal.pdf").unwrap();
    let (xref_tab, trailer) = backend.read_xref_table_and_trailer().unwrap();

    let mut search_paths = RefCell::new(Vec::new());

    support_gfx::run("hello_gfx.rs".to_owned(), CLEAR_COLOR, |ui| {
        let inspector = Inspector::new(ui, |x| backend.resolve(&xref_tab, x) );
        ui.window(im_str!("Inspect PDF"))
            .size((300.0, 100.0), ImGuiSetCond_FirstUseEver)
            .build(|| {
                inspector.draw(ui, &trailer);
            });

        // Search window
        ui.window(im_str!("Search PDF"))
            .size((300.0, 100.0), ImGuiSetCond_FirstUseEver)
            .build(|| {
                let mut search_term = ImString::with_capacity(20);
                if ui.input_text(im_str!("Search term"), &mut search_term).enter_returns_true(true).build() {
                    // Start search!
                    let mut search_paths = &mut *search_paths.borrow_mut();
                    *search_paths = inspector.search_key(&trailer.clone().into(), search_term.to_str());

                    println!("Paths: {:?}", search_paths);
                }
                for path in &*search_paths.borrow() {
                    ui.text(im_str!("{}", path_to_string(&path)));
                }
            });

        true
    });
}

fn path_to_string(path: &SearchPath) -> String {
    let mut result: String = String::new();
    for elem in path.iter() {
        match *elem {
            PathElem::DictElem {ref key} => {
                result += "->";
                result += &key
            }
            PathElem::ArrayElem {index} => {
                result += &format!("[{}]", index);
            }
        }
    }
    result
}


struct Inspector<'a, 'b: 'a, R: Resolve> {
    ui: &'a Ui<'b>,
    resolve: R ,
}

impl<'a, 'b, R: Resolve> Inspector<'a, 'b, R> {
    pub fn new(ui: &'a Ui<'b>, resolve: R) -> Inspector<'a, 'b, R> {
        Inspector {
            ui: ui,
            resolve: resolve,
        }
    }
    pub fn draw(&self, ui: &Ui, root: &Dictionary) {
        ui.text(im_str!("PDF file"));
        ui.separator();
        self.view_dict(root);
    }

    pub fn view_primitive(&self, prim: &Primitive) {
        match *prim {
            Primitive::Null => {},
            Primitive::Integer (x) => self.ui.text(im_str!("{}", x)),
            Primitive::Number (x) => self.ui.text(im_str!("{}", x)),
            Primitive::Boolean (x) => self.ui.text(im_str!("{}", x)),
            Primitive::String (ref x) => self.ui.text(im_str!("\"{}\"", x.as_str().unwrap_or("<indiscernible string>"))),
            Primitive::Stream (ref x) => {
                self.attr("Data", &PdfString::new(x.data.clone()).into(), 0);
                self.attr("Info", &x.info.clone().into(), 1);
                self.ui.tree_node(im_str!("Info")).build(|| self.view_dict(&x.info));
            }
            Primitive::Dictionary (ref x) => self.view_dict(x),
            Primitive::Array (ref x) => {
                for (i, prim) in x.iter().enumerate() {
                    let i = i as i32;
                    self.attr(&format!("elem{}", i), prim, i);
                }
            }
            Primitive::Reference (ref x) => {
                match self.resolve.resolve(*x) {
                    Ok(primitive) => {
                        self.attr(&format!("Ref[{}, {}]", x.id, x.gen), &primitive, 0);
                    }
                    Err(_) => {im_str!("<error resolving object>");},
                }
            }
            Primitive::Name (ref x) => self.ui.text(im_str!("/{}", x))
        };
    }

    pub fn view_dict(&self, dict: &Dictionary) {
        let mut id = 0;
        for (key, val) in dict.iter() {
            self.attr(key, val, id);
            id += 1;
        }
        if dict.len() == 0 {
            self.ui.text(im_str!("<No entries in dictionary>"));
        }
    }

    /// Note: the point with `id` is just that ImGui needs some unique string identifier for each
    /// tree node on the same level.
    pub fn attr(&self, name: &str, val: &Primitive, id: i32) {
        let name = im_str!("{} <{}>", name, val.get_debug_name());
        self.ui.tree_node(im_str!("{}", id)).label(name).build(|| self.view_primitive(val));
    }
}
