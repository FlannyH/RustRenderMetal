#include <metal_stblib>

using namespace metal;

// Vertex layout
typedef struct {
    packed_float2 position;
    packed_float3 color;
} vertex_t;

// Data that's passed from the vertex shader to the fragment shader
struct VertexShaderOutput {
    float4 position [[position]];
    float4 color;
};

// Vertex shader function
vertex VertexShaderOutput hello_triangle_vertex(const device vertex_t* vertex_array [[buffer(0)]], uint vertex_index [[vertex_id]]) {
    VertexShaderOutput out;
    auto device const &vertex = vertex_array[vid];
    out.color = float(vertex.color.r, vertex.color.g, vertex.color.b, 1.0);
    out.position = float4(vertex.position.x, vertex.position.y, 0.0, 1.0);
    return out;
}

// Fragment shader function
fragment float4 hello_triangle_fragment(VertexShaderOutput in [[stage_in]]) {
    return in.color;
}