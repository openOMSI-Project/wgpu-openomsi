// The vertex output struct is reused as the fragment input, so the fragment
// stage sees `@invariant @builtin(position)`. GLSL must not emit
// `invariant gl_FragCoord`, which desktop GL drivers and GLSL ES reject.
struct VertexOutput {
    @builtin(position) @invariant position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

@vertex
fn vs() -> VertexOutput {
    return VertexOutput(vec4<f32>(0.0), vec4<f32>(1.0));
}

@fragment
fn fs(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color * in.position.z;
}
