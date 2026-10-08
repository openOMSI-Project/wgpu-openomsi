// A ray query initialized again in every loop iteration must be traced again
// each time, even after an earlier traversal finished: initialization resets the
// query's initialization tracker instead of adding to it.
enable wgpu_ray_query;

@group(0) @binding(0)
var acc_struct: acceleration_structure;

@group(0) @binding(1)
var<storage, read_write> output: u32;

@compute @workgroup_size(1)
fn main() {
    var rq: ray_query;
    var start = 0.0;
    var hits = 0u;
    for (var i = 0u; i < 4u; i++) {
        rayQueryInitialize(&rq, acc_struct, RayDesc(RAY_FLAG_FORCE_OPAQUE, 0xFFu, start, 100.0, vec3<f32>(0.0), vec3<f32>(0.0, 1.0, 0.0)));
        rayQueryProceed(&rq);
        let intersection = rayQueryGetCommittedIntersection(&rq);
        if (intersection.kind == RAY_QUERY_INTERSECTION_NONE) {
            break;
        }
        hits += 1u;
        start = intersection.t + 0.001;
    }
    output = hits;
}
