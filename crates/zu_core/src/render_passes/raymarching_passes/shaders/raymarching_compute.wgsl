@group(0) @binding(0) var output_texture: texture_storage_2d<rgba8unorm, read_write>;

@group(1) @binding(0) var<storage, read> objects: array<RaymarchingObject>;

struct RaymarchingObject {
    position: vec4<f32>,
    material: i32,
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


    var ray_origin = constants.ray_origin;
    var ray_direction = normalize(vec3<f32>(uv * rot2d(constants.rotation) * constants.FOV, 1.0));
    let yz = ray_direction.yz * rot2d(constants.yz_rotation);
    ray_direction = normalize(vec3<f32>(ray_direction.x, yz));

    var distance_traveled = 0.0;
    var color = vec3<f32>(0.0);
    var normal = vec3<f32>(0.0);
    var material = 0;
    for (var i = 0; i < 80; i++) {
        let new_ray_position = ray_origin + ray_direction * distance_traveled;
        let res = map(new_ray_position);
        let distance = res.res;
        distance_traveled += distance;

        if distance < 0.05 || distance_traveled > 100.0 {

            normal = get_normal(new_ray_position);
            ray_origin = new_ray_position;
            if distance_traveled > 100.0 {
                 material = -1;
            } else {
                material = res.material;
            }

            break;
        }
    }

    // let diff = max(0.0, dot(normal, normalize(constants.sun_dir)));

    // color = diff * constants.sun_color + vec3<f32>(0.1, 0.1, 0.1);
    switch material {
        case 0: {
            color = light(normal, ray_origin, ray_direction, vec3<f32>(1.0), 1.0);
        }
        case 2: {
            let ran = hash33(ray_origin * 5.0);
            color = light(normal, ray_origin, ray_direction, ran, 1.0);
        }
        case  -1: {
            color = get_sky(ray_direction) + get_sun(ray_direction);
        }
        default: {
            color = light(normal, ray_origin, ray_direction, vec3<f32>(1.0, 0.0, 0.0), 0.5);
        }
    }

    textureStore(output_texture, vec2<i32>(pixelCoord), vec4(color, 1.0));
}

fn hash33(p: vec3<f32>) -> vec3<f32> {
    var q = vec3<f32>(
        dot(p, vec3<f32>(127.1, 311.7, 74.7)),
        dot(p, vec3<f32>(269.5, 183.3, 246.1)),
        dot(p, vec3<f32>(113.5, 271.9, 124.6))
    );
    return fract(sin(q) * 43758.5453);
}


fn get_sky(ray_direction: vec3<f32>) -> vec3<f32> {
    let cosTheta = dot(ray_direction, normalize(constants.sun_dir));
    let scatter = pow(1.0 - cosTheta, 0.5);
    let skyColor = mix(vec3(0.2,0.4,0.8), vec3(1.0,0.6,0.2), scatter);
    return skyColor;
}

fn get_sun(ray_direction: vec3<f32>) -> vec3<f32>
{
    let sunAmount = pow(clamp(dot(ray_direction, normalize(constants.sun_dir)), 0.0, 1.0), 20);
    return vec3(1.0,0.6,0.05) * sunAmount;
}

fn disney_diffuse(
    normal: vec3<f32>,
    view_dir: vec3<f32>,
    base_color: vec3<f32>,
    roughness: f32
) -> vec3<f32> {

    let N = normal;
    let V = view_dir;
    let L = normalize(constants.sun_dir);
    let H = normalize(L + V);

    let cos_theta_l = max(dot(N, L), 0.0);
    let cos_theta_v = max(dot(N, V), 0.0);
    let cos_theta_d = max(dot(L, H), 0.0);

    let fd90 = 0.5 + 2.0 * roughness * cos_theta_d * cos_theta_d;

    let light_scatter = 1.0 + (fd90 - 1.0) * pow(1.0 - cos_theta_l, 5.0);
    let view_scatter  = 1.0 + (fd90 - 1.0) * pow(1.0 - cos_theta_v, 5.0);

    let diffuse = base_color / PI;

    return diffuse * light_scatter * view_scatter * cos_theta_l;
}

fn light(
    normal: vec3<f32>,
    ray_origin: vec3<f32>,
    ray_dir: vec3<f32>,
    base_color: vec3<f32>,
    roughness: f32,
) -> vec3<f32> {
     let diffuse = disney_diffuse(normal, -ray_dir, base_color, roughness);
     let light_dir = normalize(constants.sun_dir);
     let ambient = 0.3;
     let shadow = soft_shadow(ray_origin + normal * 0.01, light_dir, 0.01, 50, 32);
     return ambient+diffuse * constants.sun_color * shadow;
}

fn soft_shadow(ray_origin: vec3<f32>, ray_dir: vec3<f32>, mint: f32, maxt: f32, k: f32) -> f32 {
    var res = 1.0;
    var t = mint;

    for(var i=0; i<256 && t < maxt; i++) {
        let h = map(ray_origin + ray_dir*t).res;
        if (h<0.001) {return 0.0;}
        res = min(res, k*h/t);
        t += h;
    }

    return res;
}


fn map(new_ray_position: vec3<f32>) -> SdfData {
    var res = SdfData(sdSphere(new_ray_position, 0.1), 0);
    for (var i = 0u; i < constants.objects_count; i = i + 1u) {
        var object = objects[i];
        res = sdf_union(res, SdfData(sdSphere(new_ray_position - object.position.xyz, object.position.w), object.material));
    }
    res = sdf_union(res, SdfData(infinite_cubes(new_ray_position), 0));

    let ground = new_ray_position.y + 0.75;
    res = sdf_union(SdfData(ground, 0), res);
    return res;
}

struct SdfData {
    res: f32,
    material: i32
}

fn sdf_union(sdf_1: SdfData, sdf_2: SdfData) -> SdfData {
    if sdf_1.res > sdf_2.res {
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
    if (new_ray_position.y > 3.0) {
        return 1000.0;
    }

    q = repeat(q, cell_size);

    return sdRoundBox(q, vec3<f32>(0.5), 0.2);
}

fn get_normal(p: vec3<f32>) -> vec3<f32> {
    let eps = 0.001;
    let nx = map(p + vec3<f32>(eps,0,0)).res - map(p - vec3<f32>(eps,0,0)).res;
    let ny = map(p + vec3<f32>(0,eps,0)).res - map(p - vec3<f32>(0,eps,0)).res;
    let nz = map(p + vec3<f32>(0,0,eps)).res - map(p - vec3<f32>(0,0,eps)).res;
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
