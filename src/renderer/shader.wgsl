struct GlobalParams {
  viewport_size: vec2<f32>,
  premulatiplied_aplha: u32,
}

@group(0)
@binding(0)
var<uniform> globals: GlobalParams;

struct Bounds {
  origin: vec2<f32>,
  size: vec2<f32>,
}

struct Corners {
  top_left: f32,
  top_right: f32,
  bottom_right: f32,
  bottom_left: f32,
}

struct Edges {
  top: f32,
  right: f32,
  bottom: f32,
  left: f32,
}

struct Hsla {
  h: f32,
  s: f32,
  l: f32,
  a: f32,
}

fn to_device_position_impl(position: vec2<f32>) -> vec4<f32> {
  let to_device_position = position / globals.viewport_size * vec2<f32>(2.0, -2.0) + vec2<f32>(-1.0, 1.0);
  return vec4<f32>(to_device_position, 0.0, 1.0);
}

fn to_device_position(unit_vertex: vec2<f32>, bounds: Bounds) -> vec4<f32> {
  let position = unit_vertex * vec2<f32>(bounds.size) + bounds.origin;
  return to_device_position_impl(position);
}

fn srgb_to_linear(srgb: vec3<f32>) -> vec3<f32> {
  let cutoff = srgb < vec3<f32>(0.04045);
  let higher = pow((srgb + vec3<f32>(0.055)) / vec3<f32>(1.055), vec3<f32>(2.4));
  let lower = srgb / vec3<f32>(12.92);
  return select(higher, lower, cutoff);
}

fn hsla_to_rgba(hsla: Hsla) -> vec4<f32> {
  let h = hsla.h * 6.0;
  let s = hsla.s;
  let l = hsla.l;
  let a = hsla.a;

  let c = (1.0 - abs(2.0 * l - 1.0)) * s;
  let x = c * (1.0 - abs(h % 2.0 -1.0));
  let m = l - c / 2.0;
  var color = vec3<f32>(m);

  if h >= 0.0 && h < 1.0 {
    color.r += c;
    color.g += x;
  } else if h >= 1.0 && h < 2.0 {
    color.r += x;
    color.g += c;
  } else if h >= 2.0 && h < 3.0 {
    color.g += c;
    color.b += x;
  } else if h >= 3.0 && h < 4.0 {
    color.g += x;
    color.b += c;
  } else if h >= 4.0 && h < 5.0 {
    color.r += x;
    color.b += c;
  } else {
    color.r += c;
    color.b += x;
  }

  let linear = srgb_to_linear(color);
  return vec4<f32>(linear, a);
}

fn over(below: vec4<f32>, above: vec4<f32>) -> vec4<f32> {
  let alpha = above.a + below.a * (1.0 - above.a);
  let color = (above.rgb * above.a + below.rgb * below.a * (1.0 - above.a)) / alpha;
  return vec4<f32>(color, alpha);
}

fn blend_color(color: vec4<f32>, alpha_factor: f32) -> vec4<f32> {
  let alpha = color.a * alpha_factor;
  let multiplier = select(1.0, alpha, globals.premulatiplied_aplha != 0u);
  return vec4<f32>(color.rgb * multiplier, alpha);
}

fn pick_corner_radius(point: vec2<f32>, radii: Corners) -> f32 {
  if point.x < 0.0 {
    if point.y < 0.0 {
      return radii.top_left;
    } else {
      return radii.bottom_left;
    }
  } else {
    if point.y < 0.0 {
      return radii.top_right;
    } else {
      return radii.bottom_right;
    }
  }
}

struct Quad {
  order: u32,
  bounds: Bounds,
  background: Hsla,
  border_color: Hsla,
  corner_radii: Corners,
  border_widths: Edges,
}

@group(0)
@binding(1)
var<storage, read> b_quads: array<Quad>;

struct QuadVarying {
  @builtin(position) position: vec4<f32>,
  @location(0) @interpolate(flat) background_color: vec4<f32>,
  @location(1) @interpolate(flat) border_color: vec4<f32>,
  @location(2) @interpolate(flat) quad_id: u32,
}

@vertex
fn vs_quad(@builtin(vertex_index) vertex_id: u32, @builtin(instance_index) instance_id: u32) -> QuadVarying {
  let unit_vertex = vec2<f32>(f32(vertex_id & 1u), 0.5 * f32(vertex_id & 2u));
  let quad = b_quads[instance_id];

  var out = QuadVarying();
  out.position = to_device_position(unit_vertex, quad.bounds);
  out.background_color = hsla_to_rgba(quad.background);
  out.border_color = hsla_to_rgba(quad.border_color);
  out.quad_id = instance_id;
  return out;
}

@fragment
fn fs_quad(input: QuadVarying) -> @location(0) vec4<f32> {
  let quad = b_quads[input.quad_id];
  if quad.corner_radii.top_left == 0.0 && quad.corner_radii.top_right == 0.0 && quad.corner_radii.bottom_right == 0.0 && quad.corner_radii.bottom_left == 0.0 && quad.border_widths.top == 0.0 && quad.border_widths.left == 0.0 && quad.border_widths.right == 0.0 && quad.border_widths.bottom == 0.0 {
    return blend_color(input.background_color, 1.0);
  }

  let half_size = quad.bounds.size / 2.0;
  let center = quad.bounds.origin + half_size;
  let center_to_point = input.position.xy - center;

  let corner_radius = pick_corner_radius(center_to_point, quad.corner_radii);

  let rounded_edge_to_point = abs(center_to_point) - half_size + corner_radius;
  let distance = length(max(vec2<f32>(0.0), rounded_edge_to_point)) + min(0.0, max(rounded_edge_to_point.x, rounded_edge_to_point.y)) - corner_radius;
  
  let vertical_border = select(quad.border_widths.left, quad.border_widths.right, center_to_point.x > 0.0);
  let horizontal_border = select(quad.border_widths.top, quad.border_widths.bottom, center_to_point.y > 0.0);
  let inset_size = half_size - corner_radius - vec2<f32>(vertical_border, horizontal_border);
  let point_to_inset_corner = abs(center_to_point) - inset_size;

  var border_width = 0.0;
  if point_to_inset_corner.x < 0.0 && point_to_inset_corner.y < 0.0 {
    border_width = 0.0;
  } else if point_to_inset_corner.y > point_to_inset_corner.x {
    border_width = horizontal_border;
  } else {
    border_width = vertical_border;
  }

  var color = input.background_color;
  if border_width > 0.0 {
    let inset_distance = distance + border_width;
    let blended_border = over(input.background_color, input.border_color);
    color = mix(blended_border, input.background_color, saturate(0.5 - inset_distance));
  }

  return blend_color(color, saturate(0.5 - distance));
}
