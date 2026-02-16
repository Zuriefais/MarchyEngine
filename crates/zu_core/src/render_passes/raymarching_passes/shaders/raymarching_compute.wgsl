@group(0) @binding(0) var output_texture: texture_storage_2d<rgba32float, read_write>;

@group(1) @binding(0) var<storage, read> objects: array<RaymarchingObject>;

struct RaymarchingObject {
    position: vec4<f32>,
    material: f32,
    pad: vec3<f32>,
}

struct PushConstants {
    texture_size: vec2<f32>,
    time: f32,
    rotation: f32,
    ray_origin: vec3<f32>,
    pad0: f32,
    FOV: f32,
    objects_count: u32,
    yz_rotation: f32,
    pad1: f32,
    sun_dir: vec3<f32>,
    pad2: f32,
    sun_color: vec3<f32>,
    pad3: f32,
};

var<push_constant> constants: PushConstants;

const PI: f32 = 3.14159265359f;


@compute @workgroup_size(32, 32)
fn compute_main(@builtin(global_invocation_id) id: vec3<u32>) {
    let pixelCoord = vec2<f32>(id.xy);
    let aspect =  constants.texture_size.x / constants.texture_size.y;
    var uv = (pixelCoord / constants.texture_size) * 2.0 - vec2<f32>(1.0, 1.0);
    uv.y = -uv.y;
    uv.x *= aspect;


    let ray_origin = constants.ray_origin;
    var ray_direction = normalize(vec3<f32>(uv * rot2d(constants.rotation) * constants.FOV, 1.0));
    let yz = ray_direction.yz * rot2d(constants.yz_rotation);
    ray_direction = normalize(vec3<f32>(ray_direction.x, yz));

    var distance_traveled = 0.0;
    var color = vec3<f32>(0.0);
    var normal = vec3<f32>(0.0);
    var material = 0.0;
    for (var i = 0; i < 80; i++) {
        let new_ray_position = ray_origin + ray_direction * distance_traveled;
        let res = map(new_ray_position);
        let distance = res.x;
        distance_traveled += distance;

        if distance < 0.05 || distance_traveled > 100.0 {
            normal = get_normal(new_ray_position);
            material = res.y;
            break;
        }
    }

    // let diff = max(0.0, dot(normal, normalize(constants.sun_dir)));

    // color = diff * constants.sun_color + vec3<f32>(0.1, 0.1, 0.1);
    if material == 0.0 {
        color = brdf(normal, vec3<f32>(1.0));
    } else {
        color = brdf(normal, vec3<f32>(1.0, 0.0, 0.0));
    }

    textureStore(output_texture, vec2<i32>(pixelCoord), vec4(color, 1.0));
}


fn brdf(normal: vec3<f32>, view_dir: vec3<f32>, base_color: vec3<f32>, roughness: f32) -> vec3<f32> {
    let light_dir = normalize(constants.sun_dir);
    let cos_theta_l = max(0.0, dot(normal, light_dir));
    let cos_theta_d = max(0.0, dot(normal, view_dir));

    let diff = max(0.0, dot(normal, light_dir));
    let fd90 = 0.5 + 2 * roughness * cos_theta_d;
    let normalized_base_color = (base_color/PI);
    return normalized_base_color * (diff * constants.sun_color + vec3<f32>(0.1,0.1,0.1));
}

fn map(new_ray_position: vec3<f32>) -> vec2<f32> {
    var res = vec2<f32>(sdSphere(new_ray_position, 0.1), 0);
    for (var i = 0u; i < constants.objects_count; i = i + 1u) {
        var object = objects[i];
        res = sdf_union(res, vec2<f32>(sdSphere(new_ray_position - object.position.xyz, object.position.w), object.material));
    }
    res = sdf_union(res, vec2<f32>(infinite_cubes(new_ray_position), 0.0));
    let ground = new_ray_position.y + 0.75;
    res = sdf_union(vec2<f32>(ground, 0), res);
    return res;
}

fn sdf_union(sdf_1: vec2<f32>, sdf_2: vec2<f32>) -> vec2<f32> {
    if sdf_1.x > sdf_2.x {
        return sdf_2;
    } else {
        return sdf_1;
    }
}

fn infinite_cubes(new_ray_position: vec3<f32>) -> f32 {
    let cell_size = 4.0;
    var q = new_ray_position;

    // q.x -= sin(constants.time);
    // q.y -= cos(constants.time);
    // q *= sin(constants.time/4);
    q = repeat(q, cell_size);

    return sdRoundBox(q, vec3<f32>(0.5), 0.2);
}

fn get_normal(p: vec3<f32>) -> vec3<f32> {
    let eps = 0.001;
    let nx = map(p + vec3<f32>(eps,0,0)).x - map(p - vec3<f32>(eps,0,0)).x;
    let ny = map(p + vec3<f32>(0,eps,0)).x - map(p - vec3<f32>(0,eps,0)).x;
    let nz = map(p + vec3<f32>(0,0,eps)).x - map(p - vec3<f32>(0,0,eps)).x;
    return normalize(vec3<f32>(nx, ny, nz));
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
