extern crate gfx;
extern crate gfx_window_glutin;
extern crate glutin;
#[macro_use]
extern crate imgui;
extern crate imgui_gfx_renderer;

extern crate pdf;

use pdf::*;
use object::*;
use file::*;

use imgui::*;

mod support_gfx;

const CLEAR_COLOR: [f32; 4] = [0.2, 0.2, 0.3, 1.0];

struct Inspector<'a, 'b: 'a> {
    pdf: &'a File<Vec<u8>>,
    ui: &'a Ui<'b>,
    unique_id: i32,
}

impl<'a, 'b> Inspector<'a, 'b> {
    pub fn new(pdf: &'a File<Vec<u8>>, ui: &'a Ui<'b>) -> Inspector<'a, 'b> {
        Inspector {
            pdf: pdf,
            ui: ui,
            unique_id: 0,
        }
    }
    pub fn draw(&mut self, ui: &Ui) {
        ui.text(im_str!("Root"));
        ui.separator();
        ui.tree_node(im_str!("{}", self.unique_id)).label(im_str!("Root")).build(|| self.pdf.get_root().view(self));
        self.unique_id += 1;
    }
}

impl<'a, 'b> Viewer for Inspector<'a, 'b> {
    // Mostly leaf nodes
    fn text(&mut self, s: &str) {
        self.ui.text(im_str!("{}", s));
    }
    // Attributes of a dictionary
    fn attr<F: Fn(&mut Self)>(&mut self, name: &str, view: F) {
        let ui = self.ui;
        ui.tree_node(im_str!("{}", self.unique_id)).label(im_str!("{}", name)).build(|| view(self));
        self.unique_id += 1;
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
    let pdf = File::<Vec<u8>>::open("files/libreoffice.pdf").unwrap();

    support_gfx::run("hello_gfx.rs".to_owned(), CLEAR_COLOR, |ui| {
        let mut inspector = Inspector::new(&pdf, ui);
        ui.window(im_str!("Inspect PDF"))
            .size((300.0, 100.0), ImGuiSetCond_FirstUseEver)
            .build(|| {
                inspector.draw(ui);
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
