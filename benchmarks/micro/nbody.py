# Benchmark: N-body simulation (5 bodies, 50000 steps).
# Measures: floating-point arithmetic, list access, nested loops.

PI: float = 3.141592653589793
SOLAR_MASS: float = 4.0 * PI * PI
DAYS_PER_YEAR: float = 365.24

# [x, y, z, vx, vy, vz, mass]
BODIES: list = [
    [0.0, 0.0, 0.0,
     0.0, 0.0, 0.0,
     SOLAR_MASS],
    [4.84143144246472090e+00, -1.16032004402742839e+00, -1.03622044471123109e-01,
     1.66007664274403694e-03 * DAYS_PER_YEAR,
     7.69901118419740425e-03 * DAYS_PER_YEAR,
     -6.90460016972063023e-05 * DAYS_PER_YEAR,
     9.54791938424326609e-04 * SOLAR_MASS],
    [8.34336671824457987e+00, 4.12479856412430479e+00, -4.03523417114321381e-01,
     -2.76742510726862411e-03 * DAYS_PER_YEAR,
     4.99852801234917238e-03 * DAYS_PER_YEAR,
     2.30417297573763929e-05 * DAYS_PER_YEAR,
     2.85885980666130812e-04 * SOLAR_MASS],
    [1.28943695621391310e+01, -1.51111514016986312e+01, -2.23307578892655734e-01,
     2.96460137564761618e-03 * DAYS_PER_YEAR,
     2.37847173959480950e-03 * DAYS_PER_YEAR,
     -2.96589568540237556e-05 * DAYS_PER_YEAR,
     4.36624404335156298e-05 * SOLAR_MASS],
    [1.53796971148509165e+01, -2.59193146099879641e+01, 1.79258772950371181e-01,
     2.68067772490389322e-03 * DAYS_PER_YEAR,
     1.62824170038242295e-03 * DAYS_PER_YEAR,
     -9.51592254519715870e-05 * DAYS_PER_YEAR,
     5.15138902046611451e-05 * SOLAR_MASS],
]


def advance(bodies: list, dt: float, n: int) -> None:
    for _step in range(n):
        for i in range(len(bodies)):
            for j in range(i + 1, len(bodies)):
                bi = bodies[i]
                bj = bodies[j]
                dx: float = bi[0] - bj[0]
                dy: float = bi[1] - bj[1]
                dz: float = bi[2] - bj[3]
                dist2: float = dx * dx + dy * dy + dz * dz
                mag: float = dt / (dist2 * (dist2 ** 0.5))
                mi: float = bi[6]
                mj: float = bj[6]
                bi[3] -= dx * mj * mag
                bi[4] -= dy * mj * mag
                bi[5] -= dz * mj * mag
                bj[3] += dx * mi * mag
                bj[4] += dy * mi * mag
                bj[5] += dz * mi * mag
        for b in bodies:
            b[0] += dt * b[3]
            b[1] += dt * b[4]
            b[2] += dt * b[5]


def energy(bodies: list) -> float:
    e: float = 0.0
    for i in range(len(bodies)):
        bi = bodies[i]
        vx: float = bi[3]
        vy: float = bi[4]
        vz: float = bi[5]
        e += 0.5 * bi[6] * (vx * vx + vy * vy + vz * vz)
        for j in range(i + 1, len(bodies)):
            bj = bodies[j]
            dx: float = bi[0] - bj[0]
            dy: float = bi[1] - bj[1]
            dz: float = bi[2] - bj[2]
            dist: float = (dx * dx + dy * dy + dz * dz) ** 0.5
            e -= (bi[6] * bj[6]) / dist
    return e


# Offset solar system momentum
px: float = 0.0
py: float = 0.0
pz: float = 0.0
for b in BODIES:
    px += b[3] * b[6]
    py += b[4] * b[6]
    pz += b[5] * b[6]
BODIES[0][3] = -px / SOLAR_MASS
BODIES[0][4] = -py / SOLAR_MASS
BODIES[0][5] = -pz / SOLAR_MASS

print(energy(BODIES))
advance(BODIES, 0.01, 50000)
print(energy(BODIES))
