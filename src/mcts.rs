use std::arch::x86_64::_lzcnt_u64;

// fast inverse square root from Quake III Arena
// https://en.wikipedia.org/wiki/Fast_inverse_square_root
/*
fn fast_inv_sqrt(x: f32) -> f32 {
  let xhalf = 0.5 * x;
  let mut i = x.to_bits();
  i = 0x5f3759df - (i >> 1);
  let x = f32::from_bits(i);
  x * (1.5 - xhalf * x * x)
  // x * (1.5 - xhalf * x * x)
}
*/

// w = winning score, n = visits, c = exploration constant, N = parent visits
// original UCB 1: w/n + c * sqrt(ln(N) / n)
// convert ln into log2: w/n + c * sqrt(log2(N) / n * log2(e))
// flip fraction to use fast inv sqrt: w/n + c * inv_sqrt(n / log2(N))
// note that we got rid of log2(e) because it only affects C, and that's a value we plan on playing around with anyway
fn ucb1(visits: u64, score: f32, parentvisits: u64) -> f32 {
  let c = 1f32; // exploration constant, we can play around with this value later
  let n = visits as f32;
  let log2_n = unsafe { (63 - _lzcnt_u64(parentvisits)) as f32 }; // fast log2
  (score / n) + c * (log2_n / n).sqrt()
}

/* 
fn ucb1_inv(visits: u64, score: f32, parentvisits: u64) -> f32 {
  let c = 1f32; // exploration constant, we can play around with this value later
  let n = visits as f32;
  let log2_n = unsafe { (63 - _lzcnt_u64(parentvisits)) as f32 }; // fast log2
  (score / n) + c * fast_inv_sqrt(n / log2_n)
}
*/