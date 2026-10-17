use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::HashSet, fs::{self, File}, io::{Read, Write}, path::{Component, Path}};
pub type Result<T> = std::result::Result<T, String>;
pub const MAX_BYTES: u64 = 16 * 1024 * 1024 * 1024;
pub const MAX_FILES: usize = 20_000;
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entry { pub path: String, pub size: u64, pub sha256: String }
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest { pub schema_version: u32, pub installation: String, pub files: Vec<Entry> }
pub fn safe_path(s: &str) -> bool {
 !s.is_empty() && !s.contains(['\\', ':', '\0']) && !s.starts_with('/') &&
 s.split('/').all(|p| !p.is_empty() && p != "." && p != "..") &&
 Path::new(s).components().all(|p| matches!(p, Component::Normal(_)))
}
pub fn validate_manifest(m: &Manifest) -> Result<()> {
 if m.schema_version != 1 || m.installation.trim().is_empty() || m.files.is_empty() || m.files.len() > MAX_FILES { return Err("Unsupported or empty manifest".into()); }
 let mut names = HashSet::new(); let mut total=0u64;
 for e in &m.files {
  total=total.checked_add(e.size).ok_or("Size overflow")?;
  if !safe_path(&e.path) || !names.insert(e.path.to_lowercase()) || e.sha256.len()!=64 || !e.sha256.bytes().all(|b| b.is_ascii_hexdigit()) || total>MAX_BYTES { return Err("Invalid manifest entry".into()); }
 }
 Ok(())
}
pub fn hash_file(path: &Path) -> Result<String> {
 let mut f=File::open(path).map_err(|e|e.to_string())?; let mut h=Sha256::new(); let mut buf=[0u8;65536];
 loop { let n=f.read(&mut buf).map_err(|e|e.to_string())?; if n==0 {break} h.update(&buf[..n]); }
 Ok(format!("{:x}",h.finalize()))
}
pub fn verify(root: &Path, m: &Manifest) -> Result<()> {
 validate_manifest(m)?;
 for e in &m.files {
  let mut p=root.to_path_buf();
  for part in Path::new(&e.path).components() { p.push(part); let md=fs::symlink_metadata(&p).map_err(|x|x.to_string())?; if md.file_type().is_symlink() {return Err("Symbolic links are forbidden".into())} }
  let md=fs::metadata(&p).map_err(|x|x.to_string())?;
  if !md.is_file() || md.len()!=e.size || hash_file(&p)? != e.sha256.to_lowercase() { return Err(format!("Corrupted or mismatched file: {}",e.path)); }
 }
 Ok(())
}
// Destination must be a new private staging directory. Remove it on any failure.
pub fn extract_zip(archive: &Path, destination: &Path) -> Result<()> {
 fs::create_dir(destination).map_err(|e|e.to_string())?;
 let result=(|| {
  let file=File::open(archive).map_err(|e|e.to_string())?;
  let mut zip=zip::ZipArchive::new(file).map_err(|e|e.to_string())?;
  if zip.len()>MAX_FILES {return Err("Too many archive entries".into())}
  let mut names=HashSet::new(); let mut total=0u64;
  for i in 0..zip.len() {
   let mut item=zip.by_index(i).map_err(|e|e.to_string())?;
   let name=item.name().trim_end_matches('/').to_owned();
   if !safe_path(&name) || !names.insert(name.to_lowercase()) || item.unix_mode().is_some_and(|m| m & 0o170000 != 0 && m & 0o170000 != 0o100000 && m & 0o170000 != 0o040000) {return Err("Unsafe archive entry".into())}
   total=total.checked_add(item.size()).ok_or("Archive size overflow")?;
   if total>MAX_BYTES {return Err("Archive exceeds extraction limit".into())}
   let p=destination.join(name);
   if item.is_dir() {fs::create_dir_all(p).map_err(|e|e.to_string())?; continue}
   fs::create_dir_all(p.parent().ok_or("Missing parent")?).map_err(|e|e.to_string())?;
   let mut out=fs::OpenOptions::new().write(true).create_new(true).open(p).map_err(|e|e.to_string())?;
   let expected=item.size(); let n=std::io::copy(&mut (&mut item).take(expected+1), &mut out).map_err(|e|e.to_string())?;
   if n!=expected {return Err("Archive size mismatch".into())} out.flush().map_err(|e|e.to_string())?;
  }
  Ok(())
 })();
 if result.is_err() {let _=fs::remove_dir_all(destination);} result
}
#[cfg(test)] mod tests {
 use super::*;
 fn temp() -> std::path::PathBuf {let p=std::env::temp_dir().join(format!("iw4-{}-{}",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos())); fs::create_dir(&p).unwrap(); p}
 #[test] fn hashing_and_corruption() {let p=temp(); fs::write(p.join("test"),b"abc").unwrap(); let m=Manifest{schema_version:1,installation:"synthetic test".into(),files:vec![Entry{path:"test".into(),size:3,sha256:"ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad".into()}]}; verify(&p,&m).unwrap(); fs::write(p.join("test"),b"abd").unwrap(); assert!(verify(&p,&m).is_err()); fs::remove_dir_all(p).unwrap();}
 #[test] fn manifest_rejections() {let mut m=Manifest{schema_version:2,installation:"test".into(),files:vec![]}; assert!(validate_manifest(&m).is_err()); m.schema_version=1; assert!(validate_manifest(&m).is_err()); for s in ["../escape","/abs","a\\b","C:bad","a/../b","a//b"] {assert!(!safe_path(s));}}
 #[test] fn archive_traversal_and_valid_extract() {for (name,valid) in [("../escape",false),("main/test.txt",true)] {let p=temp(); let a=p.join("a.zip"); let mut w=zip::ZipWriter::new(File::create(&a).unwrap()); w.start_file(name,zip::write::SimpleFileOptions::default()).unwrap(); w.write_all(b"original fixture").unwrap(); w.finish().unwrap(); let d=p.join("out"); assert_eq!(extract_zip(&a,&d).is_ok(),valid); if valid {assert_eq!(fs::read(d.join(name)).unwrap(),b"original fixture")} else {assert!(!d.exists());assert!(!p.join("escape").exists())} fs::remove_dir_all(p).unwrap();}}
}
