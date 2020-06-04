mod gui;
use gui::Editor;

fn main() {
    let mut editor = Editor::new();
    editor.open();

    loop {
        editor.event_loop_step();
    }
}
