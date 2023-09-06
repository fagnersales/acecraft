// IT ONLY WORKS IN Z DIRECTION!!!!!!

// CHANGE THIS PATH
const PATH = [
  // CHANGE THIS FOR THE FIRST AXIS
  [15306.5, 36.0, 15223.5],
  // CHANGE THIS FOR THE OTHER AXIS
  [15367.5, 36.0, 15162.5]
]

// MULTIPLIERS
const Z_MULTIPLIER = -1;
const Z_SPACE_MULTIPLIER = -3;

// RUN WITH: node calc.js







































const [startX, endX] = [PATH[0][0], PATH[1][0]];
const [startZ, endZ] = [PATH[0][2], PATH[1][2]];

let currentY = PATH[0][1];
let currentX = startX;
let currentZ = startZ;
let current_quadrant = 0;

let itteration = 0;
let max_itterations = 0;

let biggest_z = startZ > endZ ? startZ : endZ;
let minor_z = startZ < endZ ? startZ : endZ;

let cooking = 0;

let inv = (num) => num < 0 ? num * -1 : num;

for (let i = minor_z; i <= biggest_z;) {
  if (cooking === 2) {
    i += inv(Z_MULTIPLIER);
  } else if (cooking === 4) {
    i += inv(Z_SPACE_MULTIPLIER);
    cooking = 0;
  }

  cooking += 1;
  max_itterations += 1;
}

max_itterations = max_itterations - 1;

const newPath = [];

for (let coord = 0; itteration < max_itterations; coord++) {
  if (coord === 0) (currentX = startX)
  else if (coord === 1) (currentX = endX)
  else {
    if (coord === 2) (currentX = endX, coord = 3)
    else if (coord % 2 === 0) (currentX = currentX === endX ? startX : endX)
  }

  if (current_quadrant === 2) (currentZ += Z_MULTIPLIER);
  if (current_quadrant === 4) (currentZ += Z_SPACE_MULTIPLIER, current_quadrant = 0);

  newPath.push([currentX, currentY, currentZ]);

  current_quadrant += 1;
  itteration += 1;
}

const text = `path = [\n${newPath.map(([x, y, z], i) => `  { action = "${i % 2 == 0 ? "walking" : "right_clicking"}", to = [${x}, ${y}.0, ${z}] }`).join(',\n')}\n]`

console.log({ max_itterations });
console.log(text);

// // const path = [
// //   [0, 1, 0],
// //   [20, 1, 12]
// // ]


// // const [startX, endX] = [path[0][0], path[1][0]];
// // const [startZ, endZ] = [path[0][2], path[1][2]];

// // let currentX = startX;
// // let currentZ = 0;
// // let quadrant = 0;

// // const newPath = [];

// // for (let coord = 0; currentZ <= endZ; coord++) {
// //   if (coord === 0) (currentX = startX)
// //   else if (coord === 1) (currentX = endX)
// //   else {
// //     if (coord === 2) (currentX = endX, coord = 3)
// //     else if (coord % 2 === 0) (currentX = currentX === endX ? startX : endX)
// //   }

// //   if (quadrant === 2) (currentZ += 1);
// //   if (quadrant === 4) (currentZ += 3, quadrant = 0);

// //   newPath.push([currentX, 1, currentZ]);

// //   quadrant += 1;
// // }

// // newPath.pop();

// // newPath.forEach((p, i) => console.log(`(${i + 1})`, p));
