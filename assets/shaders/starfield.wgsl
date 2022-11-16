// This shader is inspired by Start Nest by Pablo Roman Andrioli:
// https://www.shadertoy.com/view/XlfGRj

#import bevy_sprite::mesh2d_view_bindings

//
// Description : WGSL 2D simplex noise function
//      Author : Ian McEwan, Ashima Arts
//  Maintainer : ijm
//     Lastmod : 20110822 (ijm)
//     License :
//  Copyright (C) 2011 Ashima Arts. All rights reserved.
//  Distributed under the MIT License. See LICENSE file.
//  https://github.com/ashima/webgl-noise
//
// Transcribed from GLSL to WGSL by Chris Joel
fn mod289_3(x: vec3<f32>) -> vec3<f32> {
    return x - floor(x * (1. / 289.)) * 289.;
}

fn mod289_2(x: vec2<f32>) -> vec2<f32> {
    return x - floor(x * (1. / 289.)) * 289.;
}

fn permute(x: vec3<f32>) -> vec3<f32> {
    return mod289_3(((x * 34.) + 1.) * x);
}

fn snoise(v: vec2<f32>) -> f32 {
  // Precompute values for skewed triangular grid
  let C = vec4<f32>(
    .211324865405187,
    // (3.0-sqrt(3.0))/6.0
    .366025403784439,
    // 0.5*(sqrt(3.0)-1.0)
    -.577350269189626,
    // -1.0 + 2.0 * C.x
    0.024390243902439);
    // 1.0 / 41.0

    // First corner (x0)
    var i = floor(v + dot(v, C.yy));
    let x0 = v - i + dot(i, C.xx);

    // Other two corners (x1, x2)
    let i1 = select(
        vec2<f32>(0., 1.),
        vec2<f32>(1., 0.),
        x0.x > x0.y);
    let x1 = x0.xy + C.xx - i1;
    let x2 = x0.xy + C.zz;

    // Do some permutations to avoid
    // truncation effects in permutation
    i = mod289_2(i);
    let p = permute(
        permute(i.y + vec3<f32>(0., i1.y, 1.)) + i.x + vec3<f32>(0., i1.x, 1.));
    var m = max(0.5 - vec3<f32>(
        dot(x0, x0),
        dot(x1, x1),
        dot(x2, x2)
    ), vec3<f32>(0.));

    m = m * m;
    m = m * m;

    // Gradients:
    //  41 pts uniformly over a line, mapped onto a diamond
    //  The ring size 17*17 = 289 is close to a multiple
    //      of 41 (41*7 = 287)

    let x = 2. * fract(p * C.www) - 1.;
    let h = abs(x) - .5;
    let ox = floor(x + .5);
    let a0 = x - ox;

    // Normalise gradients implicitly by scaling m
    // Approximation of: m *= inversesqrt(a0*a0 + h*h);
    m = m * (1.79284291400159 - .85373472095314 * (a0 * a0 + h * h));

    // Compute final noise value at P
    let g = vec3<f32>(
        a0.x * x0.x + h.x * x0.y,
        a0.yz * vec2<f32>(x1.x, x2.x) + h.yz * vec2<f32>(x1.y, x2.y)
    );

    return 130. * dot(m, g);
}

fn rand2(p: vec2<f32>) -> vec2<f32> {
    let p = vec2<f32>(dot(p, vec2<f32>(12.9898, 78.233)), dot(p, vec2<f32>(26.65125, 83.054543)));
    return fract(sin(p) * 43758.5453);
}

fn rand(p: vec2<f32>) -> f32 {
    return fract(sin(dot(p, vec2<f32>(54.90898, 18.233))) * 4337.5453);
}

fn stars(x: vec2<f32>, num_cells: f32, size: f32, br: f32) -> f32 {
    let n = x * num_cells;
    let f = floor(n);

    var d = 1.0e10;
    for (var i = -1; i <= 1; i = i + 1) {
        for (var j = -1; j <= 1; j = j + 1) {
            var g = f + vec2<f32>(f32(i), f32(j));
			g = n - g - rand2(g % num_cells) + rand(g);
            // Control size
            g = g / (num_cells * size);
			d = min(d, dot(g, g));
        }
    }

    return br * (smoothstep(.95, 1., (1. - sqrt(d))));    
}

fn hsv2rgb(c: vec3<f32>) -> vec3<f32> {
    let K = vec4<f32>(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    let p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, vec3<f32>(0.0, 0.0, 0.0), vec3<f32>(1.0, 1.0, 1.0)), c.y);
}

fn fractal_noise(coord: vec2<f32>, persistence: f32, lacunarity: f32) -> f32
{
    let octaves = 10;
    var n = 0.0;
    var frequency = 1.0;
    var amplitude = 1.0;
    for (var o = 0; o < octaves; o = o + 1) {
        n = n + amplitude * snoise(coord * frequency);
        amplitude = amplitude * persistence;
        frequency = frequency * lacunarity;
    }
    return max(n / 5.0 - 0.2, 0.0) * 5.0;
}

fn fractal_nebula(coord: vec2<f32>, color: vec3<f32>, transparency: f32) -> vec3<f32>
{
    let n = fractal_noise(coord, .5, 2.);
    return n * color * transparency;
}

@group(1) @binding(0)
var<uniform> coords: vec2<f32>;
@group(1) @binding(1)
var<uniform> seed: f32;


struct FragmentInput {
    #import bevy_sprite::mesh2d_vertex_output
};

@fragment
fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {
    var result = vec3<f32>(0.0, 0.0, 0.0);
    let coords = vec2<f32>(-coords.x, coords.y);
    let move_factor = 50000.0;
    let time_factor = 0.05;
    
    let seed = seed;

    let nebula_color_1 = hsv2rgb(vec3<f32>(0.5 + 0.5 * sin(seed + globals.time * time_factor), 0.5, 0.25));
	let nebula_color_2 = hsv2rgb(vec3<f32>(0.5 + 0.5 * sin(seed + globals.time * time_factor * 1.5), 1.0, 0.25));
	let nebula_color_3 = hsv2rgb(vec3<f32>(0.5 + 0.5 * sin(seed + globals.time * time_factor * 2.0), 1.0, 0.25));
    
    result = result + fractal_nebula((in.uv - coords / move_factor) * 10.0 + vec2<f32>(0.025 + seed, 0.0), nebula_color_1, 0.8);
    result = result + fractal_nebula((in.uv - coords / move_factor) * 10.0  + vec2<f32>(seed, 0.025), nebula_color_2, 0.4);
    result = result + fractal_nebula((in.uv - coords / move_factor) * 10.0  + vec2<f32>(-0.025 + seed, -0.025), nebula_color_2, 0.4);

    let intensity = clamp(rand(in.uv * globals.time), 0.5, 1.0);

    result = result + stars(in.uv + vec2<f32>(seed, 0.0) - coords / (move_factor * 1.2), 5.0, 0.02, 2.0) * vec3<f32>(0.6, 0.6, 0.6) * intensity;
    result = result + stars(in.uv + vec2<f32>(seed, 0.0) - coords / (move_factor * 1.4), 15.0, 0.01, 1.0) * vec3<f32>(.7, .7, .7) * intensity;
    result = result + stars(in.uv + vec2<f32>(seed, 0.0) - coords / (move_factor * 2.0), 30.0, 0.005, 0.5) * vec3<f32>(.75, .75, .75) * intensity;

    return vec4<f32>(result, 1.0);
}
 