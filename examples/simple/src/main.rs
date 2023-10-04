use dragonfly::{FontManager, WebContext};
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

    let mut ctx = WebContext::new_from_html(
        include_str!("../../../tests/garbage.html"),
        "http://localhost",
        FontManager::with_fallback_font(),
    )
    .unwrap();
    ctx.load().await.unwrap();
}
