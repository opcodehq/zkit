// The engine's 4 × 4 Bayer threshold (`Matrix::Bayer4::threshold`), so faces
// drawn in JavaScript match the native ones cell for cell.
export function Matrix4(x, y) {
  let rank = 0;
  for (let bit = 1; bit < 4; bit *= 2) {
    const a = Math.floor(x / bit) & 1;
    const b = Math.floor(y / bit) & 1;
    rank = rank * 4 + 2 * (a ^ b) + b;
  }
  return (rank + 0.5) / 16;
}
