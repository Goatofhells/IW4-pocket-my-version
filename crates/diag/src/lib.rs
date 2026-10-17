pub fn cache_limit_mb(requested: u32, physical_mb: u64) -> u32 {
 requested.clamp(16, 512).min((physical_mb / 16).clamp(16, 512) as u32)
}
#[cfg(test)] mod tests { use super::*; #[test] fn bounded_memory() { assert_eq!(cache_limit_mb(1024, 2048),128); assert_eq!(cache_limit_mb(0, 4096),16); assert_eq!(cache_limit_mb(512,8192),512); } }
