@group(0) @binding(0) var output_texture: texture_storage_2d<rgba32float, read_write>;

struct PushConstants {
    texture_size: vec2<f32>,
    time: f32,
    rotation: f32,
    ray_origin: vec3<f32>,
    FOV: f32
};

var<push_constant> constants: PushConstants;


@compute @workgroup_size(16, 16)
fn compute_main(@builtin(global_invocation_id) id: vec3<u32>) {
    let pixelCoord = vec2<f32>(id.xy);
    let uv = (pixelCoord / constants.texture_size) * 2.0 - vec2<f32>(1.0, 1.0);


    let ray_origin = constants.ray_origin;
    let ray_direction = normalize(vec3<f32>(uv * rot2d(constants.rotation) * constants.FOV, 1.0));

    var distance_traveled = 0.0;
    var color = vec3<f32>(0.0);

    for (var i = 0; i < 80; i++) {
        let new_ray_position = ray_origin + ray_direction * distance_traveled;

        let distance = map(new_ray_position);
        distance_traveled += distance;

        if distance < 0.001 || distance_traveled > 100.0 {
            break;
        }
    }

    color = vec3(distance_traveled * .2);

    textureStore(output_texture, vec2<i32>(pixelCoord), vec4(color, 1.0));
}

fn map(new_ray_position: vec3<f32>) -> f32 {
    let box_pos = vec3<f32>(cos(constants.time), sin(constants.time), 0.0);
    let ground = new_ray_position.y + 0.75;
    let p = new_ray_position - box_pos;
    let q = vec3f(p.xy * rot2d(constants.time), p.z);
    return min(min(sdRoundBox(q, vec3<f32>(1.0), 0.5), sdSphere(new_ray_position, 1.0)), ground);
}


fn sdSphere(new_ray_position: vec3<f32>, radius: f32 ) -> f32
{
  return length(new_ray_position) - radius;
}

fn sdRoundBox(p: vec3<f32>, b: vec3<f32>, r: f32 ) -> f32
{
  let q = abs(p) - b + r;
  return length(max(q, vec3<f32>(0.0))) + min(max(q.x,max(q.y,q.z)),0.0) - r;
}

fn rot2d(angle: f32) -> mat2x2<f32> {
    let sin = sin(angle);
    let cos = cos(angle);
    return mat2x2<f32>(cos, -sin, sin, cos);
}
