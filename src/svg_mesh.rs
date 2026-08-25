// svg_mesh.rs
use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use lyon_tessellation::{
    BuffersBuilder, FillOptions, FillTessellator, FillVertex, FillVertexConstructor, VertexBuffers,
    geom::point, path::Path as LyonPath,
};

struct Ctor;
impl FillVertexConstructor<[f32; 3]> for Ctor {
    fn new_vertex(&mut self, v: FillVertex) -> [f32; 3] {
        let p = v.position();
        [p.x, -p.y, 0.0] // SVG Y points down, Bevy Y points up
    }
}

/// Tessellate every filled path in `svg` into one flat mesh.
/// The viewBox centre becomes the mesh origin; the viewBox *width* maps to
/// `width` world units. Height follows the aspect ratio.
pub fn svg_to_mesh(svg: &str, width: f32) -> Mesh {
    let tree = usvg::Tree::from_str(svg, &usvg::Options::default()).expect("bad svg");
    let size = tree.size();

    let mut builder = LyonPath::builder();
    collect(tree.root(), &mut builder);
    let path = builder.build();

    let mut buf: VertexBuffers<[f32; 3], u32> = VertexBuffers::new();
    FillTessellator::new()
        .tessellate_path(
            &path,
            &FillOptions::tolerance(0.05),
            &mut BuffersBuilder::new(&mut buf, Ctor),
        )
        .expect("tessellation failed");

    let scale = width / size.width();
    let cx = size.width() * 0.5;
    let cy = -size.height() * 0.5; // Ctor already negated Y

    let positions: Vec<[f32; 3]> = buf
        .vertices
        .iter()
        .map(|v| [(v[0] - cx) * scale, (v[1] - cy) * scale, 0.0])
        .collect();
    let n = positions.len();

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0, 0.0, 1.0]; n])
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0, 0.0]; n])
    .with_inserted_indices(Indices::U32(buf.indices))
}

fn collect(group: &usvg::Group, builder: &mut lyon_tessellation::path::path::Builder) {
    for node in group.children() {
        match node {
            usvg::Node::Group(g) => collect(g, builder),
            usvg::Node::Path(p) if p.is_visible() && p.fill().is_some() => {
                let data = p.data().clone().transform(p.abs_transform()).unwrap();
                let mut open = false;
                for seg in data.segments() {
                    use usvg::tiny_skia_path::PathSegment as S;
                    match seg {
                        S::MoveTo(a) => {
                            if open {
                                builder.end(false);
                            }
                            builder.begin(point(a.x, a.y));
                            open = true;
                        }
                        S::LineTo(a) => {
                            builder.line_to(point(a.x, a.y));
                        }
                        S::QuadTo(a, b) => {
                            builder.quadratic_bezier_to(point(a.x, a.y), point(b.x, b.y));
                        }
                        S::CubicTo(a, b, c) => {
                            builder.cubic_bezier_to(
                                point(a.x, a.y),
                                point(b.x, b.y),
                                point(c.x, c.y),
                            );
                        }
                        S::Close => {
                            builder.end(true);
                            open = false;
                        }
                    }
                }
                if open {
                    builder.end(false);
                }
            }
            _ => {}
        }
    }
}
