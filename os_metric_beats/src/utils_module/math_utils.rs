use crate::common::*;

#[doc = "소수점 둘째짜리까지만 표현해주는 함수"]
pub fn round2(x: f32) -> f32 {
    (x * 100.0).round() / 100.0
}

#[doc = "소수점 둘째짜리까지만 표현해주는 함수"]
pub fn round2_f64(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}