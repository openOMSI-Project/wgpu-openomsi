// User constants named like DXC's built-in ray tracing enum values must be
// renamed in HLSL output, otherwise they redefine the built-ins.
enable wgpu_ray_query;

const RAY_FLAG_CULL_BACK_FACING_TRIANGLES: u32 = 0x10u;
const RAY_FLAG_FORCE_NON_OPAQUE: u32 = 0x02u;
const COMMITTED_TRIANGLE_HIT: u32 = 1u;
const CANDIDATE_NON_OPAQUE_TRIANGLE: u32 = 0u;
const HIT_KIND_TRIANGLE_FRONT_FACE: u32 = 0xFEu;

@group(0) @binding(0)
var acc_struct: acceleration_structure;

@group(0) @binding(1)
var<storage, read_write> output: u32;

@compute @workgroup_size(1)
fn main() {
    var rq: ray_query;
    let flags = RAY_FLAG_CULL_BACK_FACING_TRIANGLES | RAY_FLAG_FORCE_NON_OPAQUE;
    rayQueryInitialize(&rq, acc_struct, RayDesc(flags, 0xFFu, 0.1, 100.0, vec3<f32>(0.0), vec3<f32>(0.0, 1.0, 0.0)));
    while (rayQueryProceed(&rq)) {}
    let intersection = rayQueryGetCommittedIntersection(&rq);
    var hit = CANDIDATE_NON_OPAQUE_TRIANGLE;
    if (intersection.kind == COMMITTED_TRIANGLE_HIT) {
        hit = HIT_KIND_TRIANGLE_FRONT_FACE;
    }
    output = hit;
}
