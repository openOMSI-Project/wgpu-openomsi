#version 330 core
struct VertexOutput {
    vec4 position;
    vec4 color;
};
smooth in vec4 _vs2fs_location0;
layout(location = 0) out vec4 _fs2p_location0;

void main() {
    VertexOutput in_ = VertexOutput(gl_FragCoord, _vs2fs_location0);
    _fs2p_location0 = (in_.color * in_.position.z);
    return;
}

