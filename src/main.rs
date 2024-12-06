use std::sync::Arc;

use geometry::{Bounds, Point, ScaledPixels};

mod color;
mod geometry;
mod renderer;
mod scene;

fn main() {
    let gpu_ctx = renderer::context::WgpuContext::new();
    let mut renderer = renderer::Renderer::new(Arc::new(gpu_ctx));

    let scene = scene::Scene {
        quads: vec![
            scene::Quad {
                order: 1,
                pad: 0,
                bounds: Bounds {
                    origin: geometry::Point {
                        x: geometry::ScaledPixels(10.0),
                        y: geometry::ScaledPixels(10.0),
                    },
                    size: geometry::Size {
                        width: geometry::ScaledPixels(200.0),
                        height: geometry::ScaledPixels(200.0),
                    },
                },
                background: color::Hsla::green(),
                border_color: color::Hsla::red(),
                corner_radii: geometry::Corners {
                    top_left: ScaledPixels(15.0),
                    top_right: ScaledPixels(15.0),
                    bottom_left: ScaledPixels(15.0),
                    bottom_right: ScaledPixels(15.0),
                },
                border_widths: geometry::Edges {
                    top: ScaledPixels(1.0),
                    bottom: ScaledPixels(1.0),
                    left: ScaledPixels(1.0),
                    right: ScaledPixels(1.0),
                },
            },
            scene::Quad {
                order: 0,
                pad: 0,
                bounds: Bounds {
                    origin: geometry::Point {
                        x: geometry::ScaledPixels(250.0),
                        y: geometry::ScaledPixels(10.0),
                    },
                    size: geometry::Size {
                        width: geometry::ScaledPixels(150.0),
                        height: geometry::ScaledPixels(150.0),
                    },
                },
                background: color::Hsla::green(),
                border_color: color::Hsla::red(),
                corner_radii: geometry::Corners {
                    top_left: ScaledPixels(15.0),
                    top_right: ScaledPixels(15.0),
                    bottom_left: ScaledPixels(15.0),
                    bottom_right: ScaledPixels(15.0),
                },
                border_widths: geometry::Edges {
                    top: ScaledPixels(1.0),
                    bottom: ScaledPixels(1.0),
                    left: ScaledPixels(1.0),
                    right: ScaledPixels(1.0),
                },
            },
            scene::Quad {
                order: 0,
                pad: 0,
                bounds: Bounds {
                    origin: geometry::Point {
                        x: geometry::ScaledPixels(250.0),
                        y: geometry::ScaledPixels(250.0),
                    },
                    size: geometry::Size {
                        width: geometry::ScaledPixels(150.0),
                        height: geometry::ScaledPixels(150.0),
                    },
                },
                background: color::Hsla::black(),
                border_color: color::Hsla::red(),
                corner_radii: geometry::Corners {
                    top_left: ScaledPixels(75.0),
                    top_right: ScaledPixels(75.0),
                    bottom_left: ScaledPixels(75.0),
                    bottom_right: ScaledPixels(75.0),
                },
                border_widths: geometry::Edges {
                    top: ScaledPixels(1.0),
                    bottom: ScaledPixels(1.0),
                    left: ScaledPixels(1.0),
                    right: ScaledPixels(1.0),
                },
            },
        ],
        monochrome_sprites: vec![],
    };

    renderer.draw(&scene);
}
