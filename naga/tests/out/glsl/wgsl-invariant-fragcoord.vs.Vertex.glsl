#version 330 core
struct VertexOutput {
    vec4 position;
    vec4 color;
};
invariant gl_Position;
smooth out vec4 _vs2fs_location0;

void main() {
    VertexOutput _tmp_return = VertexOutput(vec4(0.0), vec4(1.0));
    gl_Position = _tmp_return.position;
    _vs2fs_location0 = _tmp_return.color;
    return;
}

