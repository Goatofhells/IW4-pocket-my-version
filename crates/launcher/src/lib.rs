use std::{ffi::{CStr,CString},os::raw::c_char,path::Path};
fn response(value: serde_json::Value) -> *mut c_char { CString::new(value.to_string()).unwrap().into_raw() }
unsafe fn input<'a>(p:*const c_char)->Result<&'a str,String> {if p.is_null(){return Err("Null argument".into())} CStr::from_ptr(p).to_str().map_err(|e|e.to_string())}
// All strings are UTF-8, NUL-terminated, valid until the call returns.
// Every returned pointer must be freed exactly once with iw4_string_free.
#[no_mangle] pub unsafe extern "C" fn iw4_verify(root:*const c_char,catalog:*const c_char)->*mut c_char {
 let result: Result<String, String>=(|| {let root=input(root)?; let catalog=input(catalog)?;
 let manifests:Vec<validation::Manifest>=serde_json::from_str(catalog).map_err(|e|e.to_string())?;
 if manifests.is_empty(){return Err("Unsupported installation: no independently verified MW2 manifests are bundled. Imported files do not prove ownership.".into())}
 for m in manifests {if validation::verify(Path::new(root),&m).is_ok(){return Ok(m.installation)}}
 Err("Unsupported or corrupted installation: no manifest matched".into())})();
 match result {Ok(id)=>response(serde_json::json!({"ok":true,"message":format!("Files match {}. Runtime unavailable; ownership not verified.",id)})),Err(e)=>response(serde_json::json!({"ok":false,"message":e}))}
}
#[no_mangle] pub unsafe extern "C" fn iw4_extract(archive:*const c_char,dest:*const c_char)->*mut c_char {
 let r=(||validation::extract_zip(Path::new(input(archive)?),Path::new(input(dest)?)))();
 match r {Ok(())=>response(serde_json::json!({"ok":true,"message":"Imported archive; verification required"})),Err(e)=>response(serde_json::json!({"ok":false,"message":e}))}
}
#[no_mangle] pub extern "C" fn iw4_cache_limit(requested:u32,physical_mb:u64)->u32 {diag::cache_limit_mb(requested,physical_mb)}
#[no_mangle] pub unsafe extern "C" fn iw4_string_free(p:*mut c_char) {if !p.is_null(){drop(CString::from_raw(p))}}
pub fn valid_configuration(fps:u32,scale:f32)->bool {[30,60].contains(&fps) && scale.is_finite() && (0.5..=1.0).contains(&scale)}
#[cfg(test)] mod tests {use super::*; #[test] fn configuration_bounds(){assert!(valid_configuration(30,0.75));assert!(!valid_configuration(120,1.0));assert!(!valid_configuration(30,f32::NAN));} #[test] fn ffi_ownership(){unsafe{let root=CString::new(".").unwrap();let cat=CString::new("[]").unwrap();let p=iw4_verify(root.as_ptr(),cat.as_ptr());assert!(CStr::from_ptr(p).to_str().unwrap().contains("Unsupported"));iw4_string_free(p);}}}
