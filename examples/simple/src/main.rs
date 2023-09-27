use dragonfly::{Declaration, ParserMode};
// use raqote::*; // graphics library

/*
fn render_webcontext(ctx: &WebContext) {
    let nodes = ctx.layout.nodes();

    let mut dt = DrawTarget::new(512, 512);
    for node in nodes {
        dt.fill_rect(
            node.pos.x,
            node.pos.y,
            100.0,
            100.0,
            &Source::Solid(SolidSource::from_unpremultiplied_argb(255, 255, 0, 0)),
            &DrawOptions::new(),
        )
    }

    dt.write_png("out.png").unwrap();
}
*/

#[tokio::main]
async fn main() {
    env_logger::init();

    // let _ = Declaration::from_css(include_str!("../../../tests/test.css"), ParserMode::Normal);
    let _ = Declaration::from_css(
        include_str!("../../../src/internal/default.css"),
        ParserMode::DefaultCss,
    );
}
