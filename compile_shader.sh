# Temporary shader build script
xcrun -sdk macosx metal -c ./metal/shaders/hello_triangle.metal -o ./metal/shaders/hello_triangle.air
xcrun -sdk macosx metallib ./metal/shaders/hello_triangle.air -o ./metal/shaders/hello_triangle.metallib
