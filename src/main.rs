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
        ui.text(im_str!("Root"));
        ui.separator();
        ui.tree_node(im_str!("{}", self.new_id())).label(im_str!("Root")).build(|| self.view_dict(root));
    }

    fn view_primitive(&mut self, prim: &Primitive) {
        match *prim {
            Primitive::Null => self.ui.text(im_str!("<null>")),
            Primitive::Integer (x) => self.ui.text(im_str!("{}", x)),
            Primitive::Number (x) => self.ui.text(im_str!("{}", x)),
            Primitive::Boolean (x) => self.ui.text(im_str!("{}", x)),
            Primitive::String (ref x) => self.ui.text(im_str!("{}", "some string..")),
            Primitive::Stream (ref x) => {
                self.attr("Data", &PdfString::new(x.data.clone()).into());
                self.attr("Info", &x.info.clone().into());
                self.ui.tree_node(im_str!("{}", self.new_id())).label(im_str!("Info")).build(|| self.view_dict(&x.info));
            }
            Primitive::Dictionary (ref x) => self.view_dict(x),
            Primitive::Array (ref x) => {}
            Primitive::Reference (ref x) => {
                match self.resolve.resolve(*x) {
                    Ok(primitive) => {
                        self.attr("", &primitive);
                        // self.view_primitive(&primitive);
                    }
                    Err(_) => {im_str!("<error resolvind object>");},
                }
            }
            Primitive::Name (ref x) => self.ui.text(im_str!("{}", x))
        };
    }

    // TODO ensure that they all get the same ID every frame...

    fn view_dict(&mut self, dict: &Dictionary) {
        for (key, val) in dict.iter() {
            self.attr(key, val);
        }
    }

    fn attr(&mut self, name: &str, val: &Primitive) {
        let name = im_str!("{} <{}>", name, val.get_debug_name());
        self.ui.tree_node(im_str!("{}", self.new_id())).label(name).build(|| self.view_primitive(val));
    }
}


// Ideas
// Perhaps I need a more general way, which allows 'inspectors' to  view PDF in different ways.
// For example an inspector that shows the decoded contents of stuff.
// Or.. one that writes back the PDF file but with all streams decoded
//
// I would like to perhaps see what kind of primitive something is.
// 
// What about e.g '<not present>'? Should this kind of behaviour be delegated to the Viewer?
//   - that is, just another function empty()
//
// Objects: object(), not just pass-through like earlier, but takes the primitive type
//
// Arrays? Probably a new tree_node for each element. attr("elem1", |viewer| elem1.view(viewer))
//
// References - should we pass around a Resolve?
//
// 
// ..... I think perhaps it would be better with just to_primitive in Object:
//   - easy to get the type of Primitive
// or even easier: just read the first Primitive and use Resolve

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
        true
    });
}


    /*
    let tree = Tree {left: Some(5), right: Some(4)};
    ui.window(im_str!("Inspect PDF"))
        .size((300.0, 100.0), ImGuiSetCond_FirstUseEver)
        .build(|| {
            ui.text(im_str!("<text>"));
            ui.separator();
            ui.tree_node(im_str!("Hello")).build(|| {
                ui.tree_node(im_str!("Hello2")).build(|| {});
            });
            ui.tree_node(im_str!("Hello3")).build(|| {});
        });
    */
