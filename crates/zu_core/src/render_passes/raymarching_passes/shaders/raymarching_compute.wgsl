@group(0) @binding(0) var output_texture: texture_storage_2d<rgba32float, read_write>;

@group(1) @binding(0) var<storage, read> objects: array<RaymarchingObject>;

struct RaymarchingObject {
    position: vec3<f32>,

    radius: f32

}

struct PushConstants {
    texture_size: vec2<f32>,
    time: f32,
    rotation: f32,
    ray_origin: vec3<f32>,
    FOV: f32,
    objects_count: u32,
    yz_rotation: f32
};

var<push_constant> constants: PushConstants;


@compute @workgroup_size(16, 16)
fn compute_main(@builtin(global_invocation_id) id: vec3<u32>) {
    let pixelCoord = vec2<f32>(id.xy);
    let aspect =  constants.texture_size.x / constants.texture_size.y;
    var uv = (pixelCoord / constants.texture_size) * 2.0 - vec2<f32>(1.0, 1.0);
    uv.y = -uv.y;
    uv.x *= aspect;


    let ray_origin = constants.ray_origin + constants.time;
    var ray_direction = normalize(vec3<f32>(uv * rot2d(constants.rotation) * constants.FOV, 1.0));
    let yz = ray_direction.yz * rot2d(constants.yz_rotation);
    ray_direction = normalize(vec3<f32>(ray_direction.x, yz));

    var distance_traveled = 0.0;
    var color = vec3<f32>(0.0);

    for (var i = 0; i < 80; i++) {
        let new_ray_position = ray_origin + ray_direction * distance_traveled;


        let distance = map(new_ray_position, uv);
        distance_traveled += distance;

        if distance < 0.01 || distance_traveled > 100.0 {
            break;
        }
    }

    color = vec3(sin(distance_traveled), 0.5, cos(distance_traveled));

    textureStore(output_texture, vec2<i32>(pixelCoord), vec4(color, 1.0));
}

fn map(new_ray_position: vec3<f32>, uv: vec2<f32>) -> f32 {
    var map = sdSphere(new_ray_position, 0.1);
    for (var i = 0u; i < constants.objects_count; i = i + 1u) {
        var object = objects[i];
        map = min(map, sdSphere(new_ray_position - object.position, object.radius));
    }
    map = min(map, infinite_cubes(new_ray_position, uv));
    let ground = new_ray_position.y + 0.75;
    map = min(ground, map);
    return map;
}

fn infinite_cubes(new_ray_position: vec3<f32>, uv: vec2<f32>) -> f32 {
    let cell_size = 2.0;
    var q = new_ray_position;

    // q.x -= sin(constants.time);
    // q.y -= cos(constants.time);
    // q *= sin(constants.time/4);
    q = repeat(q, cell_size);

    return sdRoundBox(q, vec3<f32>(0.5), sin(constants.time));



}

fn repeat(p: vec3<f32>, c: f32) -> vec3<f32> {
    return p - c * floor((p + 0.5 * c) / c);
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
