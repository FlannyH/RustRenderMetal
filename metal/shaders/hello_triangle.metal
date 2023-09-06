#include <metal_stdlib>

using namespace metal;

// Vertex layout
struct vertex_t {
    packed_float2 position;
    //packed_float3 normal;
    //packed_float4 tangent;
    packed_float3 color;
    //packed_float2 uv0;
    //packed_float2 uv1;
};

// Data that's passed from the vertex shader to the fragment shader
struct VertexShaderOutput {
    float4 position [[position]];
    float4 color;
};

struct Rect {
    float x;
    float y;
    float w;
    float h;
};

struct Color {
    float r;
    float g;
    float b;
    float a;
};

// Vertex shader function
vertex VertexShaderOutput hello_triangle_vertex(const device vertex_t* vertex_array [[buffer(0)]], uint vertex_index [[vertex_id]]) {
    VertexShaderOutput out;
    const device vertex_t& vtx = vertex_array[vertex_index];
    out.color = float4(vtx.color.r, vtx.color.g, vtx.color.b, 1.0);
    out.position = float4(vtx.position.x, vtx.position.y, 0.0, 1.0);
    return out;
}

// Fragment shader function
fragment float4 hello_triangle_fragment(VertexShaderOutput in [[stage_in]]) {
    return in.color;
}