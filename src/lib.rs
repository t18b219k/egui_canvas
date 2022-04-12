#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
/// epaint to canvas api.
pub struct Renderer {
    context: web_sys::CanvasRenderingContext2d,
    textures: HashMap<TextureId, CanvasRenderingContext2d>,
    dpr:f64,
}

use epaint::{
    CircleShape, Color32, CubicBezierShape, ImageDelta, Mesh, PathShape, QuadraticBezierShape,
    RectShape, Shape, Stroke, TextShape,text::Glyph,TextureId,ImageData,textures::TexturesDelta
};
use std::collections::HashMap;
use std::io::Cursor;
use wasm_bindgen::JsCast;
use wasm_bindgen::__rt::IntoJsResult;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, HtmlImageElement};

impl Renderer {
    pub fn new(canvas_id: &str) -> Option<Self> {
        let doc = web_sys::window().and_then(|win| win.document());
        let canvas = doc?
            .get_element_by_id(canvas_id)?
            .dyn_into::<HtmlCanvasElement>().ok()?;

        let context = canvas
            .get_context("2d")
            .ok()??
            .dyn_into::<CanvasRenderingContext2d>()
            .ok()?;
        context.set_image_smoothing_enabled(false);
        let dpr =web_sys::window().map(|win|{win.device_pixel_ratio()}).unwrap_or(1.0);
        let rect = canvas.get_bounding_client_rect();
        canvas.set_width((rect.width()*dpr)as u32);
        canvas.set_height((rect.height()*dpr)as u32);
        Some(Self {
            context,
            textures: HashMap::new(),
            dpr
        })
    }
    pub fn new_with_canvas(canvas: &HtmlCanvasElement) -> Option<Self> {
        let context = canvas
            .get_context("2d")
            .ok()??
            .dyn_into::<CanvasRenderingContext2d>()
            .ok()?;
        let dpr =web_sys::window().map(|win|{win.device_pixel_ratio()}).unwrap_or(1.0);
        let rect = canvas.get_bounding_client_rect();
        canvas.set_width((rect.width()*dpr)as u32);
        canvas.set_height((rect.height()*dpr)as u32);
        Some(Self {
            context,
            textures: Default::default(),
            dpr
        })
    }
    fn paint_shape(&mut self, shape: &epaint::Shape) {
        match shape {
            Shape::Noop => {}
            Shape::Vec(shapes) => {
                for shape in shapes {
                    self.paint_shape(&shape);
                }
            }
            Shape::Circle(circle) => {
                let CircleShape {
                    center,
                    radius,
                    fill,
                    stroke,
                } = circle;
                let fill_color = format!(
                    "#{:02x}{:02x}{:02x}{:02x}",
                    fill.r(),
                    fill.g(),
                    fill.b(),
                    fill.a()
                );
                let Stroke { width, color } = stroke;
                let stroke_color = format!(
                    "#{:02x}{:02x}{:02x}{:02x}",
                    color.r(),
                    color.g(),
                    color.b(),
                    color.a()
                );
                self.context.set_line_width(*width as f64);
                self.context
                    .arc(
                        center.x as f64,
                        center.y as f64,
                        *radius as f64,
                        0.0,
                        2.0 * std::f64::consts::PI,
                    )
                    .unwrap();
                self.context
                    .set_stroke_style(&stroke_color.into_js_result().unwrap());
                self.context
                    .set_fill_style(&fill_color.into_js_result().unwrap());
                self.context.stroke();
                self.context.fill();
            }
            Shape::LineSegment { points, stroke } => {
                self.context.begin_path();
                self.context.move_to(points[0].x as f64, points[0].y as f64);
                let Stroke { width, color } = stroke;
                let color_text = format!(
                    "#{:02x}{:02x}{:02x}{:02x}",
                    color.r(),
                    color.g(),
                    color.b(),
                    color.a()
                );
                self.context.set_line_width(*width as f64);
                self.context
                    .set_stroke_style(&color_text.into_js_result().unwrap());
                self.context.line_to(points[1].x as f64, points[1].y as f64);
                self.context.stroke();
            }
            Shape::Path(p) => {
                let PathShape {
                    points,
                    closed,
                    fill,
                    stroke,
                } = p;

                let Stroke { width, color } = stroke;
                self.context.begin_path();
                self.context.set_line_width(*width as f64);
                let color_text = format!(
                    "#{:02x}{:02x}{:02x}{:02x}",
                    color.r(),
                    color.g(),
                    color.b(),
                    color.a()
                );
                self.context
                    .set_stroke_style(&color_text.into_js_result().unwrap());
                if points.len() > 1 {
                    self.context.move_to(points[0].x as f64, points[0].y as f64);
                }
                for point in points.iter().skip(1) {
                    self.context.line_to(point.x as f64, point.y as f64);
                }
                if *closed {
                    let color_text = format!(
                        "#{:02x}{:02x}{:02x}{:02x}",
                        fill.r(),
                        fill.g(),
                        fill.b(),
                        fill.a()
                    );
                    self.context
                        .set_fill_style(&color_text.into_js_result().unwrap());
                    self.context.close_path();
                    self.context.fill();
                }
                self.context.stroke();
            }
            Shape::Rect(rect) => {
                let RectShape {
                    rect,
                    rounding,
                    fill,
                    stroke,
                } = rect;
                let Stroke { width, color } = stroke;
                self.context.begin_path();
                self.context.set_line_width(*width as f64);
                let color_text = format!(
                    "#{:02x}{:02x}{:02x}{:02x}",
                    color.r(),
                    color.g(),
                    color.b(),
                    color.a()
                );
                self.context
                    .set_stroke_style(&color_text.into_js_result().unwrap());
                let color_text = format!(
                    "#{:02x}{:02x}{:02x}{:02x}",
                    fill.r(),
                    fill.g(),
                    fill.b(),
                    fill.a()
                );
                self.context
                    .set_fill_style(&color_text.into_js_result().unwrap());
                //最初の角に移動する
                let start_x = rect.min.x + rounding.nw;
                let start_y = rect.min.y;
                let next_x = rect.max.x - rounding.ne;
                self.context.move_to(start_x as f64, start_y as f64);
                self.context.line_to(next_x as f64, start_y as f64);
                self.context
                    .arc(
                        next_x as f64,
                        (rect.min.y + rounding.ne) as f64,
                        rounding.ne as f64,
                        -std::f64::consts::FRAC_PI_2,
                        0.0,
                    )
                    .unwrap();
                self.context
                    .line_to(rect.max.x as f64, (rect.max.y - rounding.se) as f64);
                self.context
                    .arc(
                        (rect.max.x - rounding.se) as f64,
                        (rect.max.y - rounding.se) as f64,
                        rounding.se as f64,
                        0.0,
                        std::f64::consts::FRAC_PI_2,
                    )
                    .unwrap();
                self.context
                    .line_to((rect.min.x + rounding.sw) as f64, rect.max.y as f64);
                self.context
                    .arc(
                        (rect.min.x + rounding.sw) as f64,
                        (rect.max.y - rounding.sw) as f64,
                        rounding.sw as f64,
                        std::f64::consts::FRAC_PI_2,
                        std::f64::consts::PI,
                    )
                    .unwrap();
                self.context
                    .line_to(rect.min.x as f64, (rect.min.y + rounding.nw) as f64);
                self.context
                    .arc(
                        (rect.min.x + rounding.nw) as f64,
                        (rect.min.y + rounding.nw) as f64,
                        rounding.nw as f64,
                        std::f64::consts::PI,
                        3.0 * std::f64::consts::FRAC_PI_2,
                    )
                    .unwrap();

                self.context.fill();
                self.context.stroke();
            }
            Shape::Text(text) => {
                let TextShape {
                    pos,
                    galley,
                    underline,
                    override_text_color: _,
                    angle: _,
                } = text;
                let rows = &galley.rows;
                let origin = pos;
                for row in rows {
                    let row_rect = row.rect;
                    for glyph in row.glyphs.iter() {
                        let Glyph {
                            chr: _,
                            pos,
                            size:_ ,
                            uv_rect,
                            section_index: _,
                        } = glyph;
                        let offset = uv_rect.offset;
                        let source_top_left = uv_rect.min;
                        let sx = source_top_left[0] as f64;
                        let sy = source_top_left[1] as f64;

                        let source_bottom_right = uv_rect.max;
                        let sw = source_bottom_right[0] as f64 - sx;
                        let sh = source_bottom_right[1] as f64 - sy;
                        let dx = (pos.x + offset.x + origin.x) as f64;
                        let dy = (pos.y + offset.y + origin.y) as f64;
                        let dw = uv_rect.size.x as f64;
                        let dh = uv_rect.size.y as f64;
                        self.context.draw_image_with_html_canvas_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
                            &self.textures.get(&TextureId::Managed(0)).unwrap().canvas().unwrap(),
                            sx,
                            sy,
                            sw,
                            sh,
                            dx,
                            dy,
                            dw,
                            dh,
                        ).unwrap();
                    }
                    if *underline != Stroke::none(){
                        let lb= row_rect.left_bottom();
                        let rb = row_rect.right_bottom();
                        let line_segment= Shape::LineSegment { points: [lb,rb], stroke: *underline };
                        self.paint_shape(&line_segment);
                    }
                }
            }
            Shape::Mesh(mesh) => {
                let Mesh {
                    indices,
                    vertices,
                    texture_id: _,
                } = mesh;
                for triangle in indices.chunks(3) {
                    self.context.begin_path();
                    let vert1 = vertices[triangle[0] as usize];
                    let vert2 = vertices[triangle[1] as usize];
                    let vert3 = vertices[triangle[2] as usize];
                    self.context.move_to(vert1.pos.x as f64, vert1.pos.y as f64);
                    self.context.line_to(vert2.pos.x as f64, vert2.pos.y as f64);
                    self.context.line_to(vert3.pos.x as f64, vert3.pos.y as f64);
                    self.context.close_path();
                    self.context.fill();
                }

                log::warn!("mesh is not supported.")
            }
            Shape::QuadraticBezier(qb) => {
                let QuadraticBezierShape {
                    points,
                    closed,
                    fill,
                    stroke,
                } = qb;
                let cp1 = points[1];
                let end = points[2];
                let Stroke { width, color } = stroke;
                self.context.begin_path();
                self.context.set_line_width(*width as f64);
                let color_text = format!(
                    "#{:02x}{:02x}{:02x}{:02x}",
                    color.r(),
                    color.g(),
                    color.b(),
                    color.a()
                );
                self.context
                    .set_stroke_style(&color_text.into_js_result().unwrap());
                self.context.move_to(points[0].x as f64, points[0].y as f64);
                if *closed {
                    let color_text = format!(
                        "#{:02x}{:02x}{:02x}{:02x}",
                        fill.r(),
                        fill.g(),
                        fill.b(),
                        fill.a()
                    );
                    self.context
                        .set_fill_style(&color_text.into_js_result().unwrap());
                }
                self.context.quadratic_curve_to(
                    cp1.x as f64,
                    cp1.y as f64,
                    end.x as f64,
                    end.y as f64,
                );
                if *closed {
                    self.context.close_path();
                    self.context.fill();
                }
                self.context.stroke();
            }
            Shape::CubicBezier(cb) => {
                let CubicBezierShape {
                    points,
                    closed,
                    fill,
                    stroke,
                } = cb;
                let cp1 = points[1];
                let cp2 = points[2];
                let end = points[3];
                let Stroke { width, color } = stroke;
                self.context.begin_path();
                self.context.set_line_width(*width as f64);
                let color_text = format!(
                    "#{:02x}{:02x}{:02x}{:02x}",
                    color.r(),
                    color.g(),
                    color.b(),
                    color.a()
                );
                self.context
                    .set_stroke_style(&color_text.into_js_result().unwrap());
                self.context.move_to(points[0].x as f64, points[0].y as f64);
                if *closed {
                    let color_text = format!(
                        "#{:02x}{:02x}{:02x}{:02x}",
                        fill.r(),
                        fill.g(),
                        fill.b(),
                        fill.a()
                    );
                    self.context
                        .set_fill_style(&color_text.into_js_result().unwrap());
                }
                self.context.bezier_curve_to(
                    cp1.x as f64,
                    cp1.y as f64,
                    cp2.x as f64,
                    cp2.y as f64,
                    end.x as f64,
                    end.y as f64,
                );
                if *closed {
                    self.context.close_path();
                    self.context.fill();
                }
                self.context.stroke();
            }
        }
    }
    pub fn paint(&mut self, shape: &epaint::ClippedShape) {
        // create clip rectangle.
        let rect = shape.0;
        self.context.begin_path();
        self.context.save();
        self.context.rect(
            rect.min.x as f64,
            rect.min.y as f64,
            rect.width() as f64,
            rect.height() as f64,
        );
        self.context.clip();
        self.paint_shape(&shape.1);
        self.context.restore();
    }
    pub fn paint_and_update_texture(
        &mut self,
        shapes: &[epaint::ClippedShape],
        textures_delta: TexturesDelta,
    ) {
        let TexturesDelta { set, free } = textures_delta;
        for (id, delta) in set {
            self.set_texture(id, delta);
        }
        self.context.scale(self.dpr,self.dpr).unwrap();
        for shape in shapes {
            self.paint(shape);
        }
        for id in free {
            self.free_texture(id);
        }
        self.context.scale(1.0/self.dpr,1.0/self.dpr).unwrap();
    }
    pub fn clear(&self, color: &Color32) {
        let canvas = self.context.canvas().unwrap();
        let width = canvas.width();
        let height = canvas.height();
        self.context.rect(0.0, 0.0, width as f64, height as f64);
        let color_text = format!(
            "#{:02x}{:02x}{:02x}{:02x}",
            color.r(),
            color.g(),
            color.b(),
            color.a()
        );
        self.context
            .set_fill_style(&color_text.into_js_result().unwrap());
        self.context.fill();
    }

    pub fn set_texture(&mut self, id: TextureId, image_delta: ImageDelta) -> Option<()> {
        let ImageDelta { image, pos } = image_delta;
        let (w, h) = (image.width(), image.height());
        let sub_image = upload_texture(image);
        // get or create canvas
        let ctx = self.textures.entry(id).or_insert_with(|| {
            let canvas: HtmlCanvasElement = web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .create_element("canvas")
                .unwrap()
                .unchecked_into();
            canvas.set_width(w as u32);
            canvas.set_height(h as u32);
            canvas
                .get_context("2d")
                .unwrap()
                .unwrap()
                .dyn_into::<CanvasRenderingContext2d>()
                .unwrap()
        });
        let pos = pos.unwrap_or([0, 0]);

        {
            let ctx_c = ctx.clone();
            let sub_image_c = sub_image.clone();
            let onload_handler = wasm_bindgen::closure::Closure::wrap(Box::new(move || {
                ctx_c
                    .draw_image_with_html_image_element(&sub_image_c, pos[0] as f64, pos[1] as f64)
                    .unwrap();
            })
                as Box<dyn FnMut()>);

            sub_image.set_onload(Some(onload_handler.as_ref().unchecked_ref()));
            onload_handler.forget();
        }

        Some(())
    }

    pub fn free_texture(&mut self, id: TextureId) {
        self.textures.remove(&id);
    }
}
/// convert egui image into HtmlImageElement.
///
/// using data url.
/// * png encode
/// * base64 encode
/// * set this url to image.src
///
fn upload_texture(image: epaint::ImageData) -> HtmlImageElement {
    let size = match &image {
        ImageData::Color(color) => color.size,
        ImageData::Alpha(alpha) => alpha.size,
    };
    let mut buffer = Vec::with_capacity(size[0] * size[1] * 4);
    match image {
        ImageData::Color(color) => color
            .pixels
            .iter()
            .for_each(|pixel| buffer.extend_from_slice(&pixel.to_array())),
        ImageData::Alpha(alpha) => alpha.pixels.iter().for_each(|pixel| {
            buffer.extend_from_slice(&Color32::from_white_alpha(*pixel).to_array())
        }),
    }
    // fill buffer by each pixels
    log::debug!("uploading image");
    // create
    let rgba_image = image::RgbaImage::from_raw(size[0] as u32, size[1] as u32, buffer).unwrap();
    let mut output_buffer = Vec::new();

    rgba_image
        .write_to(
            &mut Cursor::new(&mut output_buffer),
            image::ImageFormat::Png,
        )
        .unwrap();

    let image_in_base64 = base64::encode(output_buffer);
    // we upload pixels by data url.
    let image = web_sys::HtmlImageElement::new().unwrap();
    let data_url = format!("data:image/png;base64,{}", image_in_base64);
    image.set_src(&data_url);

    image
}
