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

/// Tessellate every filled path in `svg` into one flat mesh, centred on its
/// bounding box and scaled so its longest axis spans `target_size` world units.
pub fn svg_to_mesh(svg: &str, target_size: f32) -> Mesh {
    let tree = usvg::Tree::from_str(svg, &usvg::Options::default()).expect("bad svg");

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

    let (mut min, mut max) = ([f32::MAX; 2], [f32::MIN; 2]);
    for v in &buf.vertices {
        min[0] = min[0].min(v[0]);
        min[1] = min[1].min(v[1]);
        max[0] = max[0].max(v[0]);
        max[1] = max[1].max(v[1]);
    }
    let centre = [(min[0] + max[0]) * 0.5, (min[1] + max[1]) * 0.5];
    let extent = (max[0] - min[0]).max(max[1] - min[1]).max(f32::EPSILON);
    let scale = target_size / extent;

    let positions: Vec<[f32; 3]> = buf
        .vertices
        .iter()
        .map(|v| [(v[0] - centre[0]) * scale, (v[1] - centre[1]) * scale, 0.0])
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
