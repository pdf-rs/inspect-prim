extern crate gfx;
extern crate gfx_window_glutin;
extern crate glutin;
#[macro_use]
extern crate imgui;
extern crate imgui_gfx_renderer;

extern crate pdf;

use imgui::*;
use std::str;

use pdf::*;
use object::*;
use backend::*;
use primitive::*;

mod support_gfx;

const CLEAR_COLOR: [f32; 4] = [0.2, 0.2, 0.3, 1.0];

struct Inspector<'a, 'b: 'a, R: Resolve> {
    ui: &'a Ui<'b>,
    resolve: R ,
    unique_id: i32,
}

impl<'a, 'b, R: Resolve> Inspector<'a, 'b, R> {
    pub fn new(ui: &'a Ui<'b>, resolve: R) -> Inspector<'a, 'b, R> {
        Inspector {
            ui: ui,
            resolve: resolve,
            unique_id: 0,
        }
    }
    fn new_id(&mut self) -> i32 {
        self.unique_id += 1;
        self.unique_id
    }
    fn draw(&mut self, ui: &Ui, root: &Dictionary) {
        ui.text(im_str!("PDF file"));
        ui.separator();
        self.view_dict(root);
    }

    fn view_primitive(&mut self, prim: &Primitive) {
        match *prim {
            Primitive::Null => {},
            Primitive::Integer (x) => self.ui.text(im_str!("{}", x)),
            Primitive::Number (x) => self.ui.text(im_str!("{}", x)),
            Primitive::Boolean (x) => self.ui.text(im_str!("{}", x)),
            Primitive::String (ref x) => self.ui.text(im_str!("\"{}\"", x.as_str().unwrap())),
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
                        self.attr("", &primitive, 0);
                    }
                    Err(_) => {im_str!("<error resolvind object>");},
                }
            }
            Primitive::Name (ref x) => self.ui.text(im_str!("/{}", x))
        };
    }

    fn view_dict(&mut self, dict: &Dictionary) {
        let mut id = 0;
        for (key, val) in dict.iter() {
            self.attr(key, val, id);
            id += 1;
        }
    }

    /// Note: the point with `id` is just that ImGui needs some unique string identifier for each
    /// tree node on the same level.
    fn attr(&mut self, name: &str, val: &Primitive, id: i32) {
        let name = im_str!("{} <{}>", name, val.get_debug_name());
        self.ui.tree_node(im_str!("{}", id)).label(name).build(|| self.view_primitive(val));
    }
}

fn main() {
    let backend = Vec::<u8>::open("files/libreoffice.pdf").unwrap();
    let (xref_tab, trailer) = backend.read_xref_table_and_trailer().unwrap();


    support_gfx::run("hello_gfx.rs".to_owned(), CLEAR_COLOR, |ui| {
        let mut inspector = Inspector::new(ui, |x| backend.resolve(&xref_tab, x) );
        ui.window(im_str!("Inspect PDF"))
            .size((300.0, 100.0), ImGuiSetCond_FirstUseEver)
            .build(|| {
                inspector.draw(ui, &trailer);
            });
        // ui.show_style_editor(ui.imgui().style_mut());
        true
    });
}
