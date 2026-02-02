@group(0) @binding(0) var input_texture: texture_2d<f32>;
@group(1) @binding(0) var output_texture: texture_storage_2d<rgba32float, read_write>;
@group(2) @binding(0) var<storage, read> noise_texture: array<f32>;

struct PushConstants {
    one_over_size: vec2<f32>, // Используется только для расчета uv
    texture_size: vec2<f32>, // Используется только для расчета uv
    u_offset: f32,
    _pad: f32,
};

var<push_constant> constants: PushConstants;

@compute @workgroup_size(16, 16)
fn fs_main(@builtin(global_invocation_id) id: vec3<u32>) {
    let pixelCoord = id.xy;

    // UV текущего пикселя, нужен для расчета расстояния.
    // one_over_size здесь удобнее, чем texture_size, чтобы избежать деления.
    let uv = (vec2<f32>(pixelCoord) + 0.5) * constants.one_over_size;

    var nearestSeed = vec4(-2.0);
    var nearestDist = 999999.9;

    // Получаем размеры текстуры один раз.
    let textureDim = textureDimensions(input_texture) - 1;

    // Вручную развернутый цикл для 3x3 соседей для максимальной производительности.
    // Это избавляет от накладных расходов на сам цикл.
    for (var y: f32 = -1.0; y <= 1.0; y += 1.0) {
        for (var x: f32 = -1.0; x <= 1.0; x += 1.0) {
            // ОПТИМИЗАЦИЯ #1: Прямое вычисление целочисленных координат
            let offset = vec2<i32>(round(vec2(x, y) * constants.u_offset));
            let sampleCoord = vec2<i32>(pixelCoord) + offset;

            // ОПТИМИЗАЦИЯ #2: Замена ветвления на clamp
            let clampedCoord = clamp(sampleCoord, vec2<i32>(0), vec2<i32>(textureDim));

            let sample_value = textureLoad(input_texture, clampedCoord, 0);
            let sample_seed = sample_value.xy;

            // Проверка на "пустой" сид остается
            if sample_seed.x != 0.0 || sample_seed.y != 0.0 {
                let diff = sample_seed - uv;
                let dist = dot(diff, diff); // dot(v,v) быстрее, чем length(v)
                if dist < nearestDist {
                    nearestDist = dist;
                    nearestSeed = sample_value;
                }
            }
        }
    }

    textureStore(output_texture, vec2<i32>(pixelCoord), nearestSeed);
}
