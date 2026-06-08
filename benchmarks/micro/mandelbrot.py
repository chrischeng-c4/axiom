# Benchmark: Mandelbrot set — 200x200 grid, up to 50 iterations per point.
# Measures: floating-point arithmetic, loop performance, integer counting.

def mandelbrot(cx: float, cy: float, max_iter: int) -> int:
    x: float = 0.0
    y: float = 0.0
    for i in range(max_iter):
        if x * x + y * y > 4.0:
            return i
        xtemp: float = x * x - y * y + cx
        y = 2.0 * x * y + cy
        x = xtemp
    return max_iter


width: int = 200
height: int = 200
max_iter: int = 50
total: int = 0
for py in range(height):
    for px in range(width):
        cx: float = (px / width) * 3.5 - 2.5
        cy: float = (py / height) * 2.0 - 1.0
        total += mandelbrot(cx, cy, max_iter)

print(total)
