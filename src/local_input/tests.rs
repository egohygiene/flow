//! Authored/uncompiled/unrun. Synthetic local bytes only; no provider is launched.
use super::*;

#[test]
fn exhausted_planning_budget_never_enters_a_nested_owner() {
    let entered = std::cell::Cell::new(false);
    let deadline = Instant::now().checked_sub(Duration::from_secs(1)).unwrap();
    let result = planning_scope(&crate::NeverCancelled, deadline, |_| {
        entered.set(true);
        Ok(())
    });
    assert_eq!(result.unwrap_err().code, Code::InvalidInput);
    assert!(!entered.get());
}
#[test]
fn cancellation_after_nested_preparation_cannot_return_a_plan() {
    let cancelled = std::cell::Cell::new(false);
    let signal = || cancelled.get();
    let deadline = Instant::now() + Duration::from_secs(60);
    let result = planning_scope(&signal, deadline, |guard| {
        assert!(!guard.is_cancelled());
        cancelled.set(true);
        assert!(guard.is_cancelled());
        Ok("would-be plan")
    });
    assert_eq!(result.unwrap_err().code, Code::Cancelled);
}
struct Fixture {
    root: PathBuf,
}
impl Fixture {
    fn new() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "flow-local-input-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        Self {
            root: root.canonicalize().unwrap(),
        }
    }
    fn file(&self, name: &str, bytes: &[u8]) -> NativePath {
        let path = self.root.join(name);
        fs::write(&path, bytes).unwrap();
        NativePath::from_path(&path)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}
fn request(input: Input) -> Request {
    Request {
        schema_version: "flow.local-input-request/v1".into(),
        input,
        profile: Profile::InspectOnly,
        provider_executable: None,
        run_directory: None,
        limits: Limits::default(),
    }
}
#[cfg(unix)]
#[test]
fn single_local_file_keeps_exact_bytes_unicode_and_unknown_format() {
    let f = Fixture::new();
    let input = f.file(
        "音 with spaces.mp4",
        b"not video, only explicitly supplied bytes",
    );
    let value = inspect(
        &request(Input::File {
            path: input.clone(),
        }),
        &crate::NeverCancelled,
    )
    .unwrap();
    assert_eq!(value.files[0].path, input);
    assert_eq!(
        value.files[0].sha256,
        format!(
            "{:x}",
            Sha256::digest(b"not video, only explicitly supplied bytes")
        )
    );
    assert!(!value.domain_acceptance && !value.execution_input_freeze);
    assert!(value.content_type.contains("extension_is_not_media_proof"));
}
#[cfg(unix)]
#[test]
fn collection_order_is_preserved_and_changes_identity() {
    let f = Fixture::new();
    let a = f.file("a", b"a");
    let b = f.file("b", b"b");
    let first = inspect(
        &request(Input::OrderedCollection {
            members: vec![b.clone(), a.clone()],
        }),
        &crate::NeverCancelled,
    )
    .unwrap();
    let second = inspect(
        &request(Input::OrderedCollection {
            members: vec![a, b],
        }),
        &crate::NeverCancelled,
    )
    .unwrap();
    assert_ne!(first.inventory_sha256, second.inventory_sha256);
    assert_eq!(first.files[0].ordinal, 0);
    assert_ne!(first.files[0].sha256, second.files[0].sha256);
}
#[cfg(unix)]
#[test]
fn directory_order_empty_directories_and_explicit_nonrecursive_exclusions_are_visible() {
    let f = Fixture::new();
    f.file("b", b"b");
    f.file("a", b"a");
    fs::create_dir(f.root.join("nested")).unwrap();
    let req = request(Input::Directory {
        path: NativePath::from_path(&f.root),
        recursive: true,
    });
    let first = inspect(&req, &crate::NeverCancelled).unwrap();
    assert_eq!(first.files[0].path.path().unwrap(), f.root.join("a"));
    assert_eq!(first.directories.len(), 2);
    let mut req = req;
    req.input = Input::Directory {
        path: NativePath::from_path(&f.root),
        recursive: false,
    };
    let second = inspect(&req, &crate::NeverCancelled).unwrap();
    assert_eq!(second.excluded_subtrees.len(), 1);
    assert_ne!(first.inventory_sha256, second.inventory_sha256);
}
#[cfg(unix)]
#[test]
fn source_links_and_hardlink_aliases_are_refused() {
    use std::os::unix::fs::symlink;
    let f = Fixture::new();
    let source = f.file("source", b"source");
    symlink(source.path().unwrap(), f.root.join("link")).unwrap();
    assert!(
        inspect(
            &request(Input::Directory {
                path: NativePath::from_path(&f.root),
                recursive: true
            }),
            &crate::NeverCancelled
        )
        .is_err()
    );
    fs::remove_file(f.root.join("link")).unwrap();
    fs::hard_link(source.path().unwrap(), f.root.join("alias")).unwrap();
    assert!(
        inspect(
            &request(Input::File { path: source }),
            &crate::NeverCancelled
        )
        .is_err()
    );
}
#[cfg(unix)]
#[test]
fn native_profile_never_broadens_a_single_file_or_collection_into_parent_scope() {
    let f = Fixture::new();
    let source = f.file("one", b"x");
    let mut req = request(Input::File {
        path: source.clone(),
    });
    req.profile = Profile::OptiflowReadOnly;
    req.provider_executable = Some(NativePath::from_path(Path::new("/synthetic/never-probe")));
    req.run_directory = Some(NativePath::from_path(&f.root.join("new")));
    let result = plan(&req, &crate::NeverCancelled).unwrap();
    assert_eq!(result.availability, Availability::Unavailable);
    assert!(result.command_plan.is_none());
    req.input = Input::OrderedCollection {
        members: vec![source],
    };
    assert_eq!(
        plan(&req, &crate::NeverCancelled).unwrap().availability,
        Availability::Unavailable
    );
    assert!(!f.root.join("new").exists());
}
#[cfg(unix)]
#[test]
fn bounds_and_cancellation_refuse_without_creating_output() {
    let f = Fixture::new();
    let source = f.file("one", b"four");
    let mut req = request(Input::File { path: source });
    assert_eq!(inspect(&req, &|| true).unwrap_err().code, Code::Cancelled);
    req.limits.maximum_file_bytes = 1;
    req.limits.maximum_total_bytes = 1;
    assert!(inspect(&req, &crate::NeverCancelled).is_err());
}
#[test]
fn collection_duplicates_traversal_and_unknown_install_fields_refuse() {
    let p = NativePath::from_path(Path::new("/synthetic/same"));
    assert!(
        request(Input::OrderedCollection {
            members: vec![p.clone(), p]
        })
        .validate()
        .is_err()
    );
    assert!(
        request(Input::File {
            path: NativePath::from_path(Path::new("/synthetic/../escape"))
        })
        .validate()
        .is_err()
    );
    let mut value = serde_json::to_value(request(Input::File {
        path: NativePath::from_path(Path::new("/synthetic/file")),
    }))
    .unwrap();
    value["install_provider"] = serde_json::json!(true);
    assert!(Request::from_json_slice(&serde_json::to_vec(&value).unwrap()).is_err());
}
